use crate::compute_subaccount;
use crate::divide_e8s;
use crate::guard::convert_update_guard;
use crate::guard::GuardError;
use crate::management::{burn_eusd, transfer_icp};
use crate::multiply_e8s;
use crate::state::audit::record_swap;
use crate::state::mutate_state;
use crate::state::read_state;
use crate::state::Asset;
use crate::state::LeveragePosition;
use crate::tasks::schedule_now;
use crate::tasks::TaskType;
use crate::E8S;
use candid::CandidType;
use candid::Principal;
use ic_base_types::PrincipalId;
use ic_canister_log::log;
use icrc_ledger_types::icrc1::transfer::TransferError;
use std::collections::BTreeSet;

#[derive(CandidType, Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub enum SwapError {
    ICPLedgerError(TransferError),
    EUSDLedgerError(TransferError),
    NoPriceData,
    AlreadyProcessing,
    TemporarilyUnavailable(String),
    AmountTooSmall,
}

#[derive(
    candid::CandidType,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    candid::Deserialize,
    Ord,
    PartialOrd,
)]
pub struct Swap {
    pub caller: Principal,
    pub from: Asset,
    pub from_block_index: u64,
    pub from_amount: u64,
    pub to: Asset,
    pub rate: u64,
    pub fee: u64,
    pub timestamp: u64,
}

#[derive(
    candid::CandidType, Clone, Debug, PartialEq, Eq, serde::Serialize, candid::Deserialize,
)]
pub struct SwapSuccess {
    pub from_block_index: u64,
    pub to_block_index: u64,
}

#[derive(candid::CandidType, Clone, Debug, serde::Serialize, candid::Deserialize)]
pub struct SwapArg {
    pub from_asset: Asset,
    pub to_asset: Asset,
    pub amount: u64,
}

impl From<GuardError> for SwapError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::AlreadyProcessing => Self::AlreadyProcessing,
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

pub async fn convert_icp_to_eusd(amount: u64) -> Result<u64, SwapError> {
    let caller = ic_cdk::caller();
    let _guard = convert_update_guard(caller)?;

    if read_state(|s| s.icp_prices.is_empty()) {
        return Err(SwapError::NoPriceData);
    }
    if read_state(|s| amount < s.min_amount_to_stable) {
        return Err(SwapError::AmountTooSmall);
    }

    let caller_subaccount = compute_subaccount(PrincipalId(caller), 0);
    let core_id = ic_cdk::id();
    match transfer_icp(Some(caller_subaccount), core_id, amount).await {
        Ok(from_block_index) => {
            // We can unwrap as we reject calls if we don't have any price entry
            let last_icp_price_entry = read_state(|s| s.get_last_icp_price()).unwrap();
            let protocol_fee = multiply_e8s(read_state(|s| s.fees.base_fee), amount);
            debug_assert!(amount >= protocol_fee);
            let swap = Swap {
                caller,
                from: Asset::ICP,
                to: Asset::EUSD,
                from_block_index,
                rate: last_icp_price_entry.rate,
                fee: protocol_fee,
                from_amount: amount,
                timestamp: ic_cdk::api::time(),
            };
            log!(
                crate::P1,
                "[swap]: Success swap from {} {:?} to {:?} ",
                swap.from_amount / E8S,
                swap.from,
                swap.to,
            );
            mutate_state(|s| {
                record_swap(s, swap.clone());
            });
            schedule_now(TaskType::ProcessLogic);
            Ok(from_block_index)
        }
        Err(e) => Err(SwapError::ICPLedgerError(e)),
    }
}

pub async fn convert_eusd_to_icp(amount: u64) -> Result<u64, SwapError> {
    let caller = ic_cdk::caller();
    let _guard = convert_update_guard(caller)?;

    if read_state(|s| s.icp_prices.is_empty()) {
        return Err(SwapError::NoPriceData);
    }
    if read_state(|s| amount < s.min_amount_from_stable) {
        return Err(SwapError::AmountTooSmall);
    }
    let amount_to_transfer = amount;
    match burn_eusd(caller, amount_to_transfer).await {
        Ok(eusd_block_index) => {
            // Here we can unwrap as we reject calls if we don't have any price entry
            let last_icp_price_entry = read_state(|s| s.get_last_icp_price()).unwrap();
            let last_icp_price = last_icp_price_entry.rate;
            let protocol_fee = multiply_e8s(read_state(|s| s.fees.base_fee), amount_to_transfer);
            let deposited_amount = amount_to_transfer - protocol_fee;
            let swap = Swap {
                caller,
                from: Asset::EUSD,
                to: Asset::ICP,
                from_block_index: eusd_block_index,
                rate: last_icp_price_entry.rate,
                fee: protocol_fee,
                from_amount: amount_to_transfer,
                timestamp: ic_cdk::api::time(),
            };
            log!(
                crate::P1,
                "[swap]: Success swap from {} {:?} to {:?} ",
                swap.from_amount / E8S,
                swap.from,
                swap.to,
            );
            mutate_state(|s| {
                record_swap(s, swap.clone());
            });

            let icp_amount_to_transfer: u64 = divide_e8s(deposited_amount, last_icp_price);

            assert!(icp_amount_to_transfer > protocol_fee);
            schedule_now(TaskType::ProcessLogic);
            maybe_close_leverage_position();
            Ok(eusd_block_index)
        }
        Err(e) => Err(SwapError::EUSDLedgerError(e)),
    }
}

fn maybe_close_leverage_position() {
    let icp_collateral_amount = read_state(|s| s.icp_collateral_amount);
    let covered_icp_collateral_amount = read_state(|s| s.icp_collateral_covered_amount);
    if icp_collateral_amount < covered_icp_collateral_amount {
        // We have more collateral than covered collateral
        // no need to close position.
        log!(
            crate::P1,
            "[maybe_close_leverage_position] No need to close covered amount {} collateral amount {}",
            covered_icp_collateral_amount,
            icp_collateral_amount
        );
        return;
    }
    log!(
        crate::P1,
        "[maybe_close_leverage_position] We should close need to close covered amount {} collateral amount {}",
        covered_icp_collateral_amount,
        icp_collateral_amount
    );
    // Ordered set of margin ratios
    let mut margin_map: BTreeSet<(u64, LeveragePosition)> = Default::default();
    let current_price = read_state(|s| s.get_last_icp_price().unwrap().rate);
    for (_, positions) in read_state(|s| s.leverage_positions.clone()) {
        for position in positions {
            let margin_ratio = compute_margin_ratio(
                current_price,
                position.icp_entry_price.rate,
                position.amount,
                position.covered_amount,
            );
            margin_map.insert((margin_ratio, position));
        }
    }
    let mut covered_icp_collateral_amount = covered_icp_collateral_amount;
    for (score, pos) in margin_map {
        log!(crate::P1, "[position score] {}", score);
        if icp_collateral_amount < covered_icp_collateral_amount {
            return;
        }
        covered_icp_collateral_amount -= pos.covered_amount;
        schedule_now(TaskType::CloseLeveragePosition(pos));
    }
}

// Compute the margin ratio, the hight the better
pub fn compute_margin_ratio(
    current_price: u64,
    entry_price: u64,
    amount: u64,
    covered_amount: u64,
) -> u64 {
    let total_position_value = multiply_e8s(amount + covered_amount, current_price);
    let margin = multiply_e8s(amount, entry_price);
    let equity = total_position_value - margin;
    divide_e8s(equity, margin)
}

#[test]
fn test_leverage_map() {
    let current_price = 1000_000_000;
    let entry_price = 500_000_000;
    let amount = 1_000_000_000;
    let covered_amount = 100_000_000;
    let result = compute_margin_ratio(current_price, entry_price, amount, covered_amount);
    dbg!(result * 100 / 100_000_000);
}
