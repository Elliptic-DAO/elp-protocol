use crate::state::mutate_state;
use crate::state::CoreState;
use candid::Principal;
use std::collections::BTreeSet;
use std::marker::PhantomData;

const MAX_CONCURRENT: usize = 100;

#[must_use]
pub struct TimerLogicGuard(());

impl TimerLogicGuard {
    pub fn new() -> Option<Self> {
        mutate_state(|s| {
            if s.is_timer_running {
                return None;
            }
            s.is_timer_running = true;
            Some(TimerLogicGuard(()))
        })
    }
}

impl Drop for TimerLogicGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.is_timer_running = false;
        });
    }
}

/// Guards a block from executing twice when called by the same user and from being
/// executed [MAX_CONCURRENT] or more times in parallel.
#[must_use]
pub struct Guard<PR: PendingRequests> {
    principal: Principal,
    _marker: PhantomData<PR>,
}

pub trait PendingRequests {
    fn pending_requests(state: &mut CoreState) -> &mut BTreeSet<Principal>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    AlreadyProcessing,
    TooManyConcurrentRequests,
}

impl<PR: PendingRequests> Guard<PR> {
    /// Attempts to create a new guard for the current block. Fails if there is
    /// already a pending request for the specified [principal] or if there
    /// are at least [MAX_CONCURRENT] pending requests.
    pub fn new(principal: Principal) -> Result<Self, GuardError> {
        mutate_state(|s| {
            let principals = PR::pending_requests(s);
            if principals.contains(&principal) {
                return Err(GuardError::AlreadyProcessing);
            }
            if principals.len() >= MAX_CONCURRENT {
                return Err(GuardError::TooManyConcurrentRequests);
            }
            principals.insert(principal);
            Ok(Self {
                principal,
                _marker: PhantomData,
            })
        })
    }
}

impl<PR: PendingRequests> Drop for Guard<PR> {
    fn drop(&mut self) {
        mutate_state(|s| PR::pending_requests(s).remove(&self.principal));
    }
}

pub struct PendingLiquidityUpdates;
pub struct PendingLeverageUpdates;

pub struct PendingConvertUpdates;

impl PendingRequests for PendingLiquidityUpdates {
    fn pending_requests(state: &mut CoreState) -> &mut BTreeSet<Principal> {
        &mut state.liquidity_principals_lock
    }
}

impl PendingRequests for PendingLeverageUpdates {
    fn pending_requests(state: &mut CoreState) -> &mut BTreeSet<Principal> {
        &mut state.leverage_principals_lock
    }
}

impl PendingRequests for PendingConvertUpdates {
    fn pending_requests(state: &mut CoreState) -> &mut BTreeSet<Principal> {
        &mut state.convert_principals_lock
    }
}

pub fn leverage_update_guard(p: Principal) -> Result<Guard<PendingLeverageUpdates>, GuardError> {
    Guard::new(p)
}

pub fn liquidity_update_guard(p: Principal) -> Result<Guard<PendingLiquidityUpdates>, GuardError> {
    Guard::new(p)
}

pub fn convert_update_guard(p: Principal) -> Result<Guard<PendingConvertUpdates>, GuardError> {
    Guard::new(p)
}
