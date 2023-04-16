use crate::divide_e8s;
use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::multiply_e8s;
use crate::updates::liquidity::Liquidity;
use crate::updates::swap::Swap;
use crate::E8S_FLOAT;
use candid::CandidType;
use candid::Principal;
use ic_ledger_types::Timestamp;
use serde::Serialize;
use std::fmt;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

pub mod audit;
pub mod eventlog;

const DEFAULT_MIN_AMOUNT_FROM_STABLE: u64 = 100_000_000;
const DEFAULT_MIN_AMOUNT_TO_STABLE: u64 = 100_000_000;
const DEFAULT_MIN_AMOUNT_LEVERAGE: u64 = 100_000_000;
const DEFAULT_MIN_AMOUNT_LIQUIDITY: u64 = 100_000_000;

const DEFAULT_XRC_PRINCIPAL: &str = "uf6dk-hyaaa-aaaaq-qaaaq-cai";
const DEFAULT_ICP_LEDGER_PRINCIPAL: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const DEFAULT_EUSD_LEDGER_PRINCIPAL: &str = "renrk-eyaaa-aaaaa-aaada-cai";

thread_local! {
    static __STATE: RefCell<Option<CoreState>> = RefCell::default();
}

// Like assert_eq, but returns an error instead of panicking.
macro_rules! ensure_eq {
    ($lhs:expr, $rhs:expr, $msg:expr $(, $args:expr)* $(,)*) => {
        if $lhs != $rhs {
            return Err(format!("{} ({:?}) != {} ({:?}): {}",
                               std::stringify!($lhs), $lhs,
                               std::stringify!($rhs), $rhs,
                               format!($msg $(,$args)*)));
        }
    }
}

macro_rules! ensure {
    ($cond:expr, $msg:expr $(, $args:expr)* $(,)*) => {
        if !$cond {
            return Err(format!("Condition {} is false: {}",
                               std::stringify!($cond),
                               format!($msg $(,$args)*)));
        }
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize, Ord, PartialOrd, CandidType,
)]
pub struct IcpPrice {
    // e8s ICP price rate
    pub rate: u64,
}

#[derive(candid::CandidType, serde::Deserialize, Debug, Eq, PartialEq)]
pub struct ProtocolStatus {
    // The collateral ratio of the protocol e8s.
    pub collateral_ratio: u64,
    // The ratio of collateral covered by leverage positions e8s.
    pub coverered_ratio: u64,
    // The last ICP price entry e8s.
    pub icp_price: u64,
    // Total Value Locked in the protocol e8s.
    pub tvl: u64,
    // The amount of ICP that can be covered
    // by leverage positions e8s.
    pub coverable_amount: u64,
}

#[derive(candid::CandidType, serde::Deserialize, Debug, Eq, PartialEq)]
pub struct UserData {
    pub claimable_liquidity_rewards: u64,

    pub liquidity_provided: u64,

    pub leverage_positions: Option<Vec<LeveragePosition>>,
}

#[derive(
    CandidType, Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize, Ord, PartialOrd,
)]
pub struct LeveragePosition {
    pub owner: Principal,
    pub amount: u64,
    pub covered_amount: u64,
    pub take_profit: u64,
    pub timestamp: u64,
    pub icp_entry_price: IcpPrice,
    pub deposit_block_index: u64,
    pub fee: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, Serialize, Default)]
pub struct FeesPerAction {
    pub base_fee: u64,
    pub liquidation_fee: u64,
    pub stability_fee: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, Serialize, Ord, PartialOrd, Eq)]
pub enum ConversionType {
    SendIcp(u64),
    ReceiveIcp(u64),
    MinteUSD(u64),
    BurneUSD(u64),
}

impl std::fmt::Display for ConversionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConversionType::SendIcp(amount) => {
                write!(f, "Sent ICP: {}", *amount as f64 / E8S_FLOAT)
            }
            ConversionType::ReceiveIcp(amount) => {
                write!(f, "Received ICP: {}", *amount as f64 / E8S_FLOAT)
            }
            ConversionType::MinteUSD(amount) => {
                write!(f, "Minted eUSD: {}", *amount as f64 / E8S_FLOAT)
            }
            ConversionType::BurneUSD(amount) => {
                write!(f, "Burned eUSD: {}", *amount as f64 / E8S_FLOAT)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, Serialize, Ord, PartialOrd, Eq)]
pub struct ConversionEntry {
    pub timestamp: u64,
    pub from: ConversionType,
    pub to: ConversionType,
    pub owner: Principal,
}

#[derive(
    candid::CandidType, Clone, Debug, PartialEq, serde::Deserialize, Serialize, Ord, PartialOrd, Eq,
)]
pub enum Asset {
    ICP,
    EUSD,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
pub enum Mode {
    ReadOnly,
    RestrictedTo(Vec<Principal>),
    DepositsRestrictedTo(Vec<Principal>),
    GeneralAvailability,
    NoHttpOutCalls,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::ReadOnly => write!(f, "Read-only"),
            Mode::RestrictedTo(principals) => {
                write!(f, "Restricted to:")?;
                for principal in principals {
                    write!(f, " {}", principal)?;
                }
                Ok(())
            }
            Mode::DepositsRestrictedTo(principals) => {
                write!(f, "Deposits restricted to:")?;
                for principal in principals {
                    write!(f, " {}", principal)?;
                }
                Ok(())
            }
            Mode::GeneralAvailability => write!(f, "General availability"),
            Mode::NoHttpOutCalls => write!(f, "No HTTP out calls"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, Serialize)]
pub struct CoreState {
    /// Canisters ids
    pub eusd_ledger_principal: Principal,
    pub icp_ledger_principal: Principal,
    pub xrc_principal: Principal,

    pub liquidity_provided: BTreeMap<Principal, u64>,
    pub liquidity_rewards: BTreeMap<Principal, u64>,

    pub leverage_positions: BTreeMap<Principal, BTreeSet<LeveragePosition>>,
    pub block_index_to_owner: BTreeMap<u64, Principal>,

    pub fees: FeesPerAction,

    // Map from block index to swap
    pub open_swaps: BTreeMap<u64, Swap>,

    pub icp_collateral_amount: u64,
    pub icp_liqudity_amount: u64,
    pub icp_leverage_margin_amount: u64,
    pub icp_collateral_covered_amount: u64,
    pub protocol_balance: u64,

    pub total_eusd_minted: u64,
    pub total_eusd_burned: u64,
    pub total_available_fees: u64,

    pub mode: Mode,

    // Min Amounts
    pub min_amount_to_stable: u64,
    pub min_amount_from_stable: u64,
    pub min_amount_leverage: u64,
    pub min_amount_liquidity: u64,

    // List of all the recorded icp prices.
    //
    pub icp_prices: BTreeMap<Timestamp, IcpPrice>,

    /// Guards
    pub is_timer_running: bool,
    pub liquidity_principals_lock: BTreeSet<Principal>,
    pub leverage_principals_lock: BTreeSet<Principal>,
    pub convert_principals_lock: BTreeSet<Principal>,
}

impl CoreState {
    pub fn reinit(
        &mut self,
        InitArgs {
            eusd_ledger_principal,
            xrc_principal,
            icp_ledger_principal,
            min_amount_to_stable,
            min_amount_from_stable,
            min_amount_leverage,
            min_amount_liquidity,
            mode,
        }: InitArgs,
    ) {
        self.mode = mode;
        self.eusd_ledger_principal = eusd_ledger_principal
            .unwrap_or(Principal::from_text(DEFAULT_EUSD_LEDGER_PRINCIPAL).unwrap());
        self.xrc_principal =
            xrc_principal.unwrap_or(Principal::from_text(DEFAULT_XRC_PRINCIPAL).unwrap());
        self.icp_ledger_principal = icp_ledger_principal
            .unwrap_or(Principal::from_text(DEFAULT_ICP_LEDGER_PRINCIPAL).unwrap());
        self.min_amount_to_stable = min_amount_to_stable.unwrap_or(DEFAULT_MIN_AMOUNT_TO_STABLE);
        self.min_amount_from_stable =
            min_amount_from_stable.unwrap_or(DEFAULT_MIN_AMOUNT_FROM_STABLE);
        self.min_amount_leverage = min_amount_leverage.unwrap_or(DEFAULT_MIN_AMOUNT_LEVERAGE);
        self.min_amount_liquidity = min_amount_liquidity.unwrap_or(DEFAULT_MIN_AMOUNT_LIQUIDITY);
    }

    pub fn upgrade(&mut self, UpgradeArgs {}: UpgradeArgs) {}

    pub fn finish_swap(&mut self, from_block_index: u64) {
        if let Some(swap_to_remove) = self.open_swaps.remove(&from_block_index) {
            if swap_to_remove.from == Asset::EUSD {
                self.total_eusd_burned += swap_to_remove.from_amount;
                let deposited_amount = swap_to_remove.from_amount - swap_to_remove.fee;
                let transfered_icp = crate::divide_e8s(deposited_amount, swap_to_remove.rate);
                self.icp_collateral_amount -= transfered_icp;
            } else {
                let minted_eusd = multiply_e8s(
                    swap_to_remove.from_amount - swap_to_remove.fee,
                    swap_to_remove.rate,
                );
                self.icp_collateral_amount += swap_to_remove.from_amount - swap_to_remove.fee;
                self.total_eusd_minted += minted_eusd;
            }
        }
    }

    pub fn get_last_icp_price(&self) -> Option<IcpPrice> {
        self.icp_prices
            .iter()
            .next_back()
            .map(|entry| entry.1.clone())
    }
    pub fn get_last_icp_price_timestamp(&self) -> Option<u64> {
        self.icp_prices
            .iter()
            .next_back()
            .map(|entry| entry.0.timestamp_nanos)
    }

    pub fn get_total_liquidity_amount(&self) -> u64 {
        self.liquidity_provided.values().sum()
    }

    pub fn get_tvl(&self) -> u64 {
        multiply_e8s(
            self.icp_collateral_amount + self.icp_leverage_margin_amount + self.icp_liqudity_amount,
            self.get_last_icp_price().unwrap().rate,
        )
    }

    pub fn distribute_fee(&mut self, fee: u64) {
        self.total_available_fees += fee;
        crate::updates::liquidity::distribute_protocol_rewards(self);
    }

    pub fn get_total_leverage_amount(&self) -> u64 {
        self.leverage_positions
            .values()
            .map(|positions| {
                positions
                    .iter()
                    .map(|pos| pos.amount - pos.fee)
                    .sum::<u64>()
            })
            .sum::<u64>()
    }

    pub fn get_collateral_ratio(&self) -> u64 {
        let core_tvl_e8s = read_state(|s| {
            multiply_e8s(
                self.icp_collateral_amount
                    + self.icp_leverage_margin_amount
                    + self.icp_liqudity_amount,
                s.get_last_icp_price().unwrap().rate,
            )
        });
        let diff = self
            .total_eusd_minted
            .saturating_sub(self.total_eusd_burned);
        if diff == 0 {
            // If we didn't minted any stablecoin
            // The CR is inifinite.
            return u64::MAX;
        }
        divide_e8s(
            core_tvl_e8s,
            self.total_eusd_minted
                .saturating_sub(self.total_eusd_burned),
        )
    }

    pub fn get_coverered_ratio(&self) -> u64 {
        if self.icp_collateral_amount == 0 && self.icp_collateral_covered_amount == 0 {
            return 0;
        }
        divide_e8s(
            self.icp_collateral_covered_amount,
            self.icp_collateral_amount,
        )
    }

    pub fn add_liquidity(&mut self, liquidity: &Liquidity) {
        let liquidity_to_add = liquidity.amount - liquidity.fee;
        self.icp_liqudity_amount += liquidity.amount - liquidity.fee;
        if let Some(entry_mut) = self.liquidity_provided.get_mut(&liquidity.caller) {
            *entry_mut += liquidity_to_add;
        } else {
            self.liquidity_provided
                .insert(liquidity.caller, liquidity_to_add);
        }
    }

    pub fn remove_liquidity(&mut self, liquidity: &Liquidity) {
        debug_assert!(liquidity.amount <= self.icp_liqudity_amount);
        self.icp_liqudity_amount -= liquidity.amount;
        if let Some(entry_mut) = self.liquidity_provided.get_mut(&liquidity.caller) {
            debug_assert!(*entry_mut >= liquidity.amount);
            *entry_mut -= liquidity.amount;
        } else {
            panic!("bug: removing unexistent liquidity");
        }
    }

    pub fn open_leverage_position(&mut self, leverage_position: LeveragePosition) {
        self.icp_collateral_covered_amount += leverage_position.covered_amount;
        debug_assert!(leverage_position.amount >= leverage_position.fee);
        self.icp_leverage_margin_amount += leverage_position.amount - leverage_position.fee;
        ic_cdk::println!("Margin Amount: {}", self.icp_leverage_margin_amount);
        if let Some(entry_ref) = self.leverage_positions.get_mut(&leverage_position.owner) {
            entry_ref.insert(leverage_position.clone());
        } else {
            let mut set = BTreeSet::new();
            set.insert(leverage_position.clone());
            self.leverage_positions.insert(leverage_position.owner, set);
        }
        self.block_index_to_owner.insert(
            leverage_position.deposit_block_index,
            leverage_position.owner,
        );
    }

    pub fn close_leverage_position(
        &mut self,
        leverage_position: LeveragePosition,
        last_icp_price: IcpPrice,
        protocol_fee: u64,
    ) {
        let amount_to_transfer = crate::updates::leverage::compute_cash_out_amount(
            &leverage_position,
            last_icp_price.rate,
        );

        if let Some(user_positions) = self.leverage_positions.get_mut(&leverage_position.owner) {
            user_positions.remove(&leverage_position);
            self.icp_collateral_covered_amount -= leverage_position.covered_amount;
            debug_assert!(amount_to_transfer + protocol_fee <= self.icp_leverage_margin_amount);
            if amount_to_transfer > leverage_position.amount {
                debug_assert!(
                    self.icp_collateral_amount >= amount_to_transfer - leverage_position.amount
                );
                self.icp_collateral_amount -= amount_to_transfer - leverage_position.amount;
                debug_assert!(self.icp_leverage_margin_amount >= leverage_position.amount);
                self.icp_leverage_margin_amount -= leverage_position.amount;
                ic_cdk::println!("Closing amount profit: {}", self.icp_leverage_margin_amount);
            } else {
                debug_assert!(self.icp_leverage_margin_amount >= amount_to_transfer);
                let amount_for_protocol =
                    leverage_position.amount - amount_to_transfer - leverage_position.fee;
                self.icp_leverage_margin_amount -= amount_to_transfer;
                self.icp_collateral_amount += amount_for_protocol;
                ic_cdk::println!("Closing amount (loss): {}", self.icp_leverage_margin_amount);
            }
        } else {
            panic!("Could not find block index in user's positions.");
        }

        self.block_index_to_owner
            .remove(&leverage_position.deposit_block_index);
    }

    pub fn get_leverage_position(&self, deposit_block_index: u64) -> Option<LeveragePosition> {
        let owner = self.block_index_to_owner.get(&deposit_block_index).unwrap();
        if let Some(user_positions) = self.leverage_positions.get(owner) {
            for position in user_positions {
                if position.deposit_block_index == deposit_block_index {
                    return Some(position.clone());
                }
            }
        }
        None
    }

    pub fn get_leverage_position_of(&self, principal: Principal) -> Option<Vec<LeveragePosition>> {
        self.leverage_positions
            .get(&principal)
            .map(|positions| positions.iter().cloned().collect())
    }

    pub fn get_leverage_covered_amount(&self) -> u64 {
        self.leverage_positions
            .values()
            .map(|positions| positions.iter().map(|p| p.covered_amount).sum::<u64>())
            .sum::<u64>()
    }

    pub fn get_leverage_coverable_amount(&self) -> u64 {
        self.icp_collateral_amount
            .saturating_sub(self.get_leverage_covered_amount())
    }

    /// Checks whether the internal state of the core canister matches the other state
    /// semantically (the state holds the same data, but maybe in a slightly
    /// different form).
    pub fn check_semantically_eq(&self, other: &Self) -> Result<(), String> {
        ensure_eq!(
            self.eusd_ledger_principal,
            other.eusd_ledger_principal,
            "eusd_ledger_principal does not match"
        );
        ensure_eq!(
            self.icp_ledger_principal,
            other.icp_ledger_principal,
            "icp_ledger_principal does not match"
        );
        ensure_eq!(
            self.xrc_principal,
            other.xrc_principal,
            "xrc_principal does not match"
        );
        ensure_eq!(
            self.liquidity_provided,
            other.liquidity_provided,
            "liquidity_provided does not match"
        );
        ensure_eq!(
            self.liquidity_rewards,
            other.liquidity_rewards,
            "finalized_requests do not match"
        );
        ensure_eq!(self.fees, other.fees, "fees do not match");
        ensure_eq!(self.open_swaps, other.open_swaps, "open_swaps do not match");
        ensure_eq!(
            self.icp_collateral_amount,
            other.icp_collateral_amount,
            "icp_collateral_amount do not match"
        );
        ensure_eq!(
            self.icp_liqudity_amount,
            other.icp_liqudity_amount,
            "icp_liqudity_amount do not match"
        );

        ensure_eq!(
            self.icp_leverage_margin_amount,
            other.icp_leverage_margin_amount,
            "icp_leverage_margin_amount do not match"
        );

        ensure_eq!(
            self.icp_collateral_covered_amount,
            other.icp_collateral_covered_amount,
            "icp_collateral_covered_amount do not match"
        );

        ensure_eq!(
            self.total_eusd_minted,
            other.total_eusd_minted,
            "total_eusd_minted does not match"
        );

        ensure_eq!(
            self.total_eusd_burned,
            other.total_eusd_burned,
            "total_eusd_burned does not match"
        );

        ensure_eq!(
            self.total_available_fees,
            other.total_available_fees,
            "total_available_fees does not match"
        );
        ensure_eq!(self.mode, other.mode, "mode do not match");
        // TODO find a strategy to check the new ICP prices map.
        // ensure_eq!(self.icp_prices, other.icp_prices, "icp_prices do not match");

        Ok(())
    }

    pub fn check_invariants(&self) -> Result<(), String> {
        let covered_ratio = self.get_coverered_ratio();
        ensure!(
            covered_ratio < 100_000_000,
            "The covered ratio is greater than 1, covered amount: {:?} and collateral amount: {:?}",
            self.icp_collateral_covered_amount,
            self.icp_collateral_amount,
        );

        ensure!(
            self.total_eusd_burned <= self.total_eusd_minted,
            "Inconsistent eusd: burned {}, minted: {}",
            self.total_eusd_burned,
            self.total_eusd_minted,
        );

        ensure!(
            self.get_total_liquidity_amount() == self.icp_liqudity_amount,
            "Inconsistent liquidity: sum {}, tracked: {}",
            self.get_total_liquidity_amount(),
            self.icp_liqudity_amount,
        );

        ensure!(
            self.get_total_leverage_amount() == self.icp_leverage_margin_amount,
            "Inconsistent leverage: sum {}, tracked: {}",
            self.get_total_leverage_amount(),
            self.icp_leverage_margin_amount,
        );

        Ok(())
    }
}

/// Take the current state.
///
/// After calling this function the state won't be initialized anymore.
/// Panics if there is no state.
pub fn take_state<F, R>(f: F) -> R
where
    F: FnOnce(CoreState) -> R,
{
    __STATE.with(|s| f(s.take().expect("State not initialized!")))
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut CoreState) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&CoreState) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: CoreState) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}

impl From<InitArgs> for CoreState {
    fn from(args: InitArgs) -> Self {
        Self {
            eusd_ledger_principal: args
                .eusd_ledger_principal
                .unwrap_or(Principal::from_text(DEFAULT_EUSD_LEDGER_PRINCIPAL).unwrap()),
            // We can hardcode the leder id as it will always be the same.
            icp_ledger_principal: args
                .icp_ledger_principal
                .unwrap_or(Principal::from_text(DEFAULT_ICP_LEDGER_PRINCIPAL).unwrap()),
            xrc_principal: args
                .xrc_principal
                .unwrap_or(Principal::from_text(DEFAULT_XRC_PRINCIPAL).unwrap()),

            /// All the positions of the last week
            liquidity_provided: Default::default(),
            liquidity_rewards: Default::default(),
            block_index_to_owner: Default::default(),
            leverage_positions: Default::default(),

            mode: args.mode,

            open_swaps: Default::default(),
            total_eusd_minted: 0,
            total_eusd_burned: 0,
            total_available_fees: 0,

            min_amount_to_stable: args
                .min_amount_to_stable
                .unwrap_or(DEFAULT_MIN_AMOUNT_TO_STABLE),
            min_amount_from_stable: args
                .min_amount_from_stable
                .unwrap_or(DEFAULT_MIN_AMOUNT_FROM_STABLE),
            min_amount_leverage: args
                .min_amount_leverage
                .unwrap_or(DEFAULT_MIN_AMOUNT_LEVERAGE),
            min_amount_liquidity: args
                .min_amount_liquidity
                .unwrap_or(DEFAULT_MIN_AMOUNT_LIQUIDITY),

            fees: FeesPerAction {
                base_fee: 250_000,
                liquidation_fee: 2_500_000,
                stability_fee: 0,
            },

            icp_collateral_amount: 0,
            icp_liqudity_amount: 0,
            icp_leverage_margin_amount: 0,
            icp_collateral_covered_amount: 0,
            protocol_balance: 0,
            // List of all the recorded icp prices.
            icp_prices: Default::default(),

            // Init Guards
            is_timer_running: false,
            liquidity_principals_lock: Default::default(),
            leverage_principals_lock: Default::default(),
            convert_principals_lock: Default::default(),
        }
    }
}
