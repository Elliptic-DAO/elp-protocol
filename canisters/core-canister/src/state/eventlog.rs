use crate::lifecycle::init::InitArgs;
use crate::lifecycle::upgrade::UpgradeArgs;
use crate::logs::P0;
use crate::state::CoreState;
use crate::state::IcpPrice;
use crate::state::LeveragePosition;
use crate::updates::liquidity::{Liquidity, LiquidityType};
use crate::updates::swap::{Swap, SwapSuccess};
use candid::Principal;
use ic_canister_log::log;
use ic_ledger_types::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    #[serde(rename = "init")]
    Init(InitArgs),

    #[serde(rename = "upgrade")]
    Upgrade(UpgradeArgs),

    #[serde(rename = "open_leverage_position")]
    OpenLeveragePosition(LeveragePosition),

    #[serde(rename = "close_leverage_position")]
    CloseLeveragePosition {
        /// Block Index of the transfer to open the leverage
        /// position.
        deposit_block_index: u64,
        /// The output block index is optional because
        /// it can be None if the position get liquidated.
        output_block_index: Option<u64>,
        /// The fee collected by the protocol.
        fee: u64,
        /// The timestamp at position close.
        timestamp: u64,
        /// The ICP price used to compute PnL.
        icp_price: IcpPrice,
    },

    #[serde(rename = "swap")]
    Swap(Swap),

    #[serde(rename = "swap_success")]
    SwapSuccess(SwapSuccess),

    #[serde(rename = "liquidity")]
    Liquidity(Liquidity),

    #[serde(rename = "claim_liquidity_rewards")]
    ClaimLiquidityRewards { owner: Principal },
}

#[derive(Debug)]
pub enum ReplayLogError {
    /// There are no events in the event log.
    EmptyLog,
    /// The event log is inconsistent.
    InconsistentLog(String),
}

pub fn replay(mut events: impl Iterator<Item = Event>) -> Result<CoreState, ReplayLogError> {
    let mut state = match events.next() {
        Some(Event::Init(args)) => CoreState::from(args),
        Some(evt) => {
            return Err(ReplayLogError::InconsistentLog(format!(
                "The first event is not Init: {:?}",
                evt
            )))
        }
        None => return Err(ReplayLogError::EmptyLog),
    };
    for event in events {
        log!(P0, "Replaying event : {:?}", event);
        match event {
            Event::Init(args) => {
                state.reinit(args);
            }
            Event::Upgrade(_args) => {}
            Event::OpenLeveragePosition(leverage_position) => {
                state.icp_prices.insert(
                    Timestamp {
                        timestamp_nanos: leverage_position.timestamp,
                    },
                    leverage_position.icp_entry_price.clone(),
                );
                state.open_leverage_position(leverage_position.clone());
                state.distribute_fee(leverage_position.fee);
            }
            Event::CloseLeveragePosition {
                deposit_block_index,
                output_block_index: _,
                fee,
                timestamp,
                icp_price,
            } => {
                state.icp_prices.insert(
                    Timestamp {
                        timestamp_nanos: timestamp,
                    },
                    icp_price.clone(),
                );
                if let Some(leverage_position_to_remove) =
                    state.get_leverage_position(deposit_block_index)
                {
                    state.close_leverage_position(leverage_position_to_remove, icp_price, fee);
                } else {
                    panic!("inconsistent state, cannot close leverage position");
                }
                state.distribute_fee(fee);
            }
            Event::Swap(swap) => {
                state.icp_prices.insert(
                    Timestamp {
                        timestamp_nanos: swap.timestamp,
                    },
                    IcpPrice { rate: swap.rate },
                );
                state.distribute_fee(swap.fee);
                state.open_swaps.insert(swap.from_block_index, swap);
            }
            Event::SwapSuccess(swap_success) => {
                state.finish_swap(swap_success.from_block_index);
            }
            Event::Liquidity(liquidity) => {
                match liquidity.operation_type {
                    LiquidityType::Add => state.add_liquidity(&liquidity),
                    LiquidityType::Remove => {
                        state.remove_liquidity(&liquidity);
                    }
                }
                state.distribute_fee(liquidity.fee);
            }
            Event::ClaimLiquidityRewards { owner } => {
                state.liquidity_rewards.remove(&owner);
            }
        }
    }
    Ok(state)
}
