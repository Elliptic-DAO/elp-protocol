use crate::compute_subaccount;
use crate::divide_e8s;
use crate::guard::leverage_update_guard;
use crate::guard::GuardError;
use crate::multiply_e8s;
use crate::read_state;
use crate::state::audit::record_liquidate_leverage_position;
use crate::state::mutate_state;
use crate::state::LeveragePosition;
use crate::transfer_icp;
use crate::PrincipalId;
use crate::ICP_TRANSFER_FEE;
use crate::ONE_HOUR_NANOS;
use candid::CandidType;
use icrc_ledger_types::icrc1::transfer::TransferError;

#[derive(CandidType, serde::Deserialize)]
pub struct OpenLeveragePositionArg {
    pub amount: u64,
    pub take_profit: u64,
    pub covered_amount: u64,
}

#[derive(CandidType, serde::Deserialize, Debug)]
pub enum LeveragePositionError {
    LedgerError(TransferError),
    IndexNotFound,
    AlreadyProcessing,
    AmountTooSmall,
    PositionNotFound,
    CallerNotOwner,
    NotEnoughFundsToCover,
    TemporarilyUnavailable(String),
    TooEarlyToClose,
}

impl From<GuardError> for LeveragePositionError {
    fn from(e: GuardError) -> Self {
        match e {
            GuardError::AlreadyProcessing => Self::AlreadyProcessing,
            GuardError::TooManyConcurrentRequests => {
                Self::TemporarilyUnavailable("too many concurrent requests".to_string())
            }
        }
    }
}

const MIN_LEVERAGE_AMOUNT: u64 = 10_000_000;

pub async fn open_leverage_position(
    arg: OpenLeveragePositionArg,
) -> Result<u64, LeveragePositionError> {
    let caller = ic_cdk::caller();
    let _guard = leverage_update_guard(caller)?;

    // Check if the position is not too big or too small.
    let available_coverable_amount = read_state(|s| s.get_leverage_coverable_amount());
    if arg.covered_amount > available_coverable_amount {
        return Err(LeveragePositionError::NotEnoughFundsToCover);
    } else if arg.amount < MIN_LEVERAGE_AMOUNT {
        return Err(LeveragePositionError::AmountTooSmall);
    }

    // Tranfer ICP back to main account
    let caller_subaccount = compute_subaccount(PrincipalId(caller), 0);
    let core_id = ic_cdk::id();

    match transfer_icp(Some(caller_subaccount), core_id, arg.amount).await {
        Ok(block_index) => {
            let last_icp_price = read_state(|s| s.get_last_icp_price()).unwrap();
            let protocol_fee = multiply_e8s(read_state(|s| s.fees.base_fee), arg.amount);
            let leverage_position = LeveragePosition {
                owner: caller,
                amount: arg.amount,
                take_profit: arg.take_profit,
                timestamp: ic_cdk::api::time(),
                icp_entry_price: last_icp_price,
                covered_amount: arg.covered_amount,
                deposit_block_index: block_index,
                fee: protocol_fee,
            };
            crate::mutate_state(|s| {
                crate::state::audit::record_open_leverage_position(s, leverage_position);
            });

            Ok(block_index)
        }
        Err(e) => Err(LeveragePositionError::LedgerError(e)),
    }
}

pub async fn close_leverage_position(
    deposit_block_index: u64,
) -> Result<u64, LeveragePositionError> {
    let caller = ic_cdk::caller();
    let _guard = leverage_update_guard(caller)?;

    let position_to_close = read_state(|s| s.get_leverage_position(deposit_block_index));
    if position_to_close.is_none() {
        return Err(LeveragePositionError::PositionNotFound);
    }
    let position_to_close = position_to_close.unwrap();
    if position_to_close.owner != caller {
        return Err(LeveragePositionError::CallerNotOwner);
    }
    let now = ic_cdk::api::time();
    if now < position_to_close.timestamp + ONE_HOUR_NANOS {
        return Err(LeveragePositionError::TooEarlyToClose);
    }

    let last_icp_price = read_state(|s| s.get_last_icp_price()).unwrap();
    let amount_to_transfer = compute_cash_out_amount(&position_to_close, last_icp_price.rate);
    let protocol_fee = multiply_e8s(read_state(|s| s.fees.base_fee), amount_to_transfer);
    match transfer_icp(
        None,
        caller,
        amount_to_transfer - protocol_fee - ICP_TRANSFER_FEE,
    )
    .await
    {
        Ok(output_block_index) => {
            crate::mutate_state(|s| {
                crate::state::audit::record_close_leverage_position(
                    s,
                    output_block_index,
                    deposit_block_index,
                    protocol_fee,
                    now,
                    last_icp_price,
                );
            });
            Ok(output_block_index)
        }
        Err(e) => Err(LeveragePositionError::LedgerError(e)),
    }
}

pub fn compute_pnl(position: &LeveragePosition, current_icp_price: u64) -> i64 {
    let price_ratio = divide_e8s(position.icp_entry_price.rate, current_icp_price);
    let diff = 100_000_000_i64 - price_ratio as i64;
    if diff > 0 {
        multiply_e8s(position.covered_amount, diff as u64) as i64
    } else {
        -(multiply_e8s(diff.unsigned_abs(), position.covered_amount) as i64)
    }
}

pub fn compute_cash_out_amount(position: &LeveragePosition, current_icp_price: u64) -> u64 {
    let diff = compute_pnl(position, current_icp_price);
    if diff > 0 {
        (position.amount - position.fee) + diff as u64
    } else {
        (position.amount - position.fee) - diff.unsigned_abs()
    }
}

pub async fn check_leverage_positions() {
    let last_icp_price = read_state(|s| s.get_last_icp_price().unwrap());
    for (_principal, positions) in read_state(|s| s.leverage_positions.clone()) {
        for position in positions {
            let now = ic_cdk::api::time();
            if position.take_profit <= last_icp_price.rate {
                let amount_to_transfer =
                    compute_cash_out_amount(&position.clone(), last_icp_price.rate);
                if amount_to_transfer < crate::ICP_TRANSFER_FEE {
                    continue;
                }
                let protocol_fee =
                    multiply_e8s(read_state(|s| s.fees.base_fee), amount_to_transfer);
                ic_cdk::println!(
                    "Amount to transfer: {} & fee: {}",
                    amount_to_transfer,
                    protocol_fee
                );
                match transfer_icp(None, position.owner, amount_to_transfer - protocol_fee).await {
                    Ok(output_block_index) => {
                        crate::mutate_state(|s| {
                            crate::state::audit::record_close_leverage_position(
                                s,
                                output_block_index,
                                position.deposit_block_index,
                                protocol_fee,
                                now,
                                last_icp_price.clone(),
                            );
                        });
                    }
                    Err(_error) => {}
                }
            } else if should_liquidate(position.clone(), last_icp_price.rate) {
                // TODO Add fees
                mutate_state(|s| {
                    record_liquidate_leverage_position(
                        s,
                        position.deposit_block_index,
                        0,
                        now,
                        last_icp_price.clone(),
                    )
                });
            }
        }
    }
}

fn should_liquidate(position: LeveragePosition, current_price: u64) -> bool {
    debug_assert!(position.covered_amount + position.amount != 0);
    let liquidation_ratio = divide_e8s(
        position.covered_amount,
        position.covered_amount + position.amount,
    );
    let liquidation_price = multiply_e8s(liquidation_ratio, position.icp_entry_price.rate);
    if current_price <= liquidation_price {
        return true;
    }
    false
}

#[test]
fn test_should_liquidate() {
    use crate::state::IcpPrice;
    use candid::Principal;

    let mut current_price = IcpPrice { rate: 100_000_000 };
    let initial = IcpPrice { rate: 10_000_000 };
    let leverage_positon = LeveragePosition {
        owner: Principal::anonymous(),
        amount: 10,
        take_profit: 10,
        timestamp: 0,
        fee: 100,
        covered_amount: 10,
        icp_entry_price: initial,
        deposit_block_index: 0,
    };
    let liquidate = should_liquidate(leverage_positon.clone(), current_price.rate);
    assert_eq!(liquidate, false);
    current_price = IcpPrice { rate: 1_000_000 };
    let liquidate = should_liquidate(leverage_positon, current_price.rate);
    assert_eq!(liquidate, true);
}

#[test]
fn test_pnl_computation() {
    use crate::state::IcpPrice;
    use candid::Principal;

    let leverage_position = LeveragePosition {
        owner: Principal::anonymous(),
        amount: 500_000_000,
        covered_amount: 1_000_000_000,
        take_profit: 600_000_000,
        timestamp: 0,
        icp_entry_price: IcpPrice { rate: 400_000_000 },
        deposit_block_index: 0,
        fee: 0,
    };
    // leverage 3x
    let pnl = compute_pnl(&leverage_position, 500_000_000);
    let cash_out_amount = compute_cash_out_amount(&leverage_position, 500_000_000);
    assert!(pnl == 200_000_000);
    assert!(cash_out_amount == 700_000_000);
}
