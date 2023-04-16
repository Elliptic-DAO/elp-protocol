use crate::compute_subaccount;
use crate::guard::liquidity_update_guard;
use crate::guard::GuardError;
use crate::multiply_e8s;
use crate::state::audit::record_liquidity;
use crate::state::CoreState;
use crate::state::{mutate_state, read_state};
use crate::transfer_icp;
use crate::ICP_TRANSFER_FEE;
use candid::CandidType;
use candid::{Deserialize, Principal};
use ic_base_types::PrincipalId;
use icrc_ledger_types::icrc1::transfer::TransferError;
use serde::Serialize;

#[derive(CandidType, serde::Deserialize, Debug)]
pub enum LiquidityError {
    NotEnoughLiquidity(u64),
    AlreadyProcessing,
    LedgerError(TransferError),
    TemporarilyUnavailable(String),
    NoClaimableReward,
    NoLiquidityProvided,
    AmountTooSmall,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LiquidityType {
    Add,
    Remove,
}

#[derive(candid::CandidType, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Liquidity {
    pub caller: Principal,
    pub operation_type: LiquidityType,
    pub amount: u64,
    pub block_index: u64,
    pub timestamp: u64,
    pub fee: u64,
}

impl From<GuardError> for LiquidityError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::AlreadyProcessing => Self::AlreadyProcessing,
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

pub async fn add_liquidity(amount: u64) -> Result<u64, LiquidityError> {
    let caller = ic_cdk::caller();
    let _guard = liquidity_update_guard(caller)?;

    let caller_subaccount = compute_subaccount(PrincipalId(caller), 0);
    let core_id = ic_cdk::id();
    if amount < read_state(|s| s.min_amount_liquidity) {
        return Err(LiquidityError::AmountTooSmall);
    }

    match transfer_icp(Some(caller_subaccount), core_id, amount).await {
        Ok(block_index) => {
            let transfer_amount = amount;
            let protocol_fee = multiply_e8s(read_state(|s| s.fees.base_fee), transfer_amount);
            let liquidity = Liquidity {
                caller,
                operation_type: LiquidityType::Add,
                amount: transfer_amount,
                block_index,
                timestamp: ic_cdk::api::time(),
                fee: protocol_fee,
            };
            mutate_state(|s| {
                record_liquidity(s, liquidity);
            });

            Ok(block_index)
        }
        Err(e) => Err(LiquidityError::LedgerError(e)),
    }
}

pub async fn remove_liquidity(amount: u64, to: Principal) -> Result<u64, LiquidityError> {
    let caller = ic_cdk::caller();
    let _guard = liquidity_update_guard(caller)?;

    let caller_balance = read_state(|s| s.liquidity_provided.get(&caller).cloned());
    if caller_balance.is_none() {
        return Err(LiquidityError::NoLiquidityProvided);
    }
    let caller_balance = caller_balance.unwrap();
    if amount > caller_balance {
        return Err(LiquidityError::NotEnoughLiquidity(caller_balance));
    }
    let protocol_fee = multiply_e8s(read_state(|s| s.fees.base_fee), amount);
    if amount < protocol_fee + ICP_TRANSFER_FEE {
        return Err(LiquidityError::AmountTooSmall);
    }
    let collateral_ratio = read_state(|s| s.get_collateral_ratio());
    let amount_to_withdraw = compute_liquidity_claimable(amount - protocol_fee, collateral_ratio);
    match transfer_icp(None, to, amount_to_withdraw - ICP_TRANSFER_FEE).await {
        Ok(block_index) => {
            let liq = Liquidity {
                caller,
                operation_type: LiquidityType::Remove,
                amount,
                block_index,
                timestamp: ic_cdk::api::time(),
                fee: protocol_fee,
            };
            mutate_state(|s| {
                record_liquidity(s, liq);
            });
            Ok(block_index)
        }
        Err(e) => Err(LiquidityError::LedgerError(e)),
    }
}

pub async fn claim_liquidity_rewards() -> Result<u64, LiquidityError> {
    let caller = ic_cdk::caller();
    let _guard = liquidity_update_guard(caller)?;

    match read_state(|s| s.liquidity_rewards.get(&caller).cloned()) {
        Some(claimable_amount) => match transfer_icp(None, caller, claimable_amount).await {
            Ok(block_index) => {
                mutate_state(|s| s.liquidity_rewards.remove(&caller));
                Ok(block_index)
            }
            Err(e) => Err(LiquidityError::LedgerError(e)),
        },
        None => Err(LiquidityError::NoClaimableReward),
    }
}

pub fn distribute_protocol_rewards(state: &mut CoreState) {
    let fees_amount_to_distribute = state.total_available_fees;

    let fee_distributed = build_distribute_fee(
        fees_amount_to_distribute,
        state
            .liquidity_provided
            .iter()
            .map(|(principal, amount)| (*principal, *amount))
            .collect::<Vec<(Principal, u64)>>(),
    );

    let distributed_amount = fee_distributed
        .iter()
        .map(|(_, amount)| amount)
        .sum::<u64>();

    let liquidity_provider_count = state.liquidity_provided.len();
    ic_cdk::println!(
        "Distributing {} to {} liquidity providers.",
        distributed_amount,
        liquidity_provider_count
    );
    for (principal, amount) in fee_distributed {
        if let Some(entry_ref) = state.liquidity_rewards.get_mut(&principal) {
            *entry_ref += amount;
        } else {
            state.liquidity_rewards.insert(principal, amount);
        }
    }
    assert!(distributed_amount <= fees_amount_to_distribute);
    state.total_available_fees -= distributed_amount;
}

/// Distribute a fee amount to the liquidity providers
/// The returned Vec may have some change
fn build_distribute_fee(
    fee_amount: u64,
    liquidity_provided: Vec<(Principal, u64)>,
) -> Vec<(Principal, u64)> {
    let total_liquidity_amount = liquidity_provided.iter().map(|(_p, amount)| amount).sum();

    liquidity_provided
        .iter()
        .map(|(principal, amount)| {
            (
                *principal,
                (compute_share(*amount, total_liquidity_amount) * fee_amount as f64) as u64,
            )
        })
        .collect::<Vec<(Principal, u64)>>()
}

fn compute_share(amount_provided: u64, total_amount: u64) -> f64 {
    amount_provided as f64 / total_amount as f64
}

fn compute_liquidity_claimable(amount_to_claim: u64, collateral_ratio: u64) -> u64 {
    const UPPER_COLLATERAL_RATIO: u64 = 120_000_000;
    const SLOPE: u64 = 83_333_333;
    if collateral_ratio > UPPER_COLLATERAL_RATIO {
        amount_to_claim
    } else {
        let slippage = multiply_e8s(SLOPE, collateral_ratio);
        debug_assert!(slippage <= 100_000_000);
        multiply_e8s(amount_to_claim, slippage)
    }
}

#[test]
fn test_fee_distribution() {
    let fee_to_share: u64 = 100_000;
    let user_1 =
        Principal::from_text("mffnj-4wzis-e2gtp-g2f4e-57xw4-u6k2s-wwkwq-uef2k-dnk6q-7qisk-uqe")
            .unwrap();
    let user_2 =
        Principal::from_text("rs2j3-p6zkk-hajim-ugvqg-o46i7-3w3up-apojd-h3jz2-yd4kq-cea7m-pae")
            .unwrap();
    let user_3 =
        Principal::from_text("5mezl-he62a-p3r3f-faies-3snql-td4v6-noevw-eivgc-4lgjv-3atu6-iae")
            .unwrap();
    const ONE_ICP: u64 = 100_000_000;

    let mut liquidity_provided: Vec<(Principal, u64)> = vec![];
    liquidity_provided.push((user_1, 5 * ONE_ICP));
    liquidity_provided.push((user_2, 10 * ONE_ICP));
    liquidity_provided.push((user_3, 20 * ONE_ICP));

    let fee_vec = build_distribute_fee(fee_to_share, liquidity_provided);
    dbg!(fee_vec.clone());

    assert!(fee_to_share > fee_vec.iter().map(|(_, amount)| amount).sum::<u64>());
    assert!(fee_vec[0].1 == 14285);
    assert!(fee_vec[1].1 == 28571);
    assert!(fee_vec[2].1 == 57142);
}

#[test]
fn test_slippage() {
    let user_wants_to_claim: u64 = 1_000_000_000; // 10 ICP
    let collateral_ratio: u64 = 100_000_000; // 100% CR
    let result = compute_liquidity_claimable(user_wants_to_claim, collateral_ratio);
    assert!(result == 833_333_330);
    let collateral_ratio: u64 = 140_000_000; // 140% CR
    let result = compute_liquidity_claimable(user_wants_to_claim, collateral_ratio);
    assert!(result == user_wants_to_claim);
    let collateral_ratio: u64 = 50_000_000; // 50% CR
    let result = compute_liquidity_claimable(user_wants_to_claim, collateral_ratio);
    assert!(result == 416_666_660); // 4,16 ICP
}
