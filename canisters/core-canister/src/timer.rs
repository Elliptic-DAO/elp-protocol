use crate::state::IcpPrice;
use crate::tasks::schedule_now;
use crate::updates::leverage::compute_pnl;
use crate::{management::get_exchange_rate, state::mutate_state};
use ic_ledger_types::Timestamp;
use ic_xrc_types::GetExchangeRateResult;
use scopeguard::guard;
use std::time::Duration;

fn convert_to_8_decimals(amount: u64, decimals: u32) -> u64 {
    if decimals >= 8 {
        // If there are at least 8 decimal places, divide by 10^(decimals - 8)
        // to shift the decimal point to the left.
        amount / 10u64.pow(decimals - 8)
    } else {
        // If there are fewer than 8 decimal places, multiply by 10^(8 - decimals)
        // to shift the decimal point to the right.
        amount * 10u64.pow(8 - decimals)
    }
}

pub fn timer() {
    use crate::tasks::{pop_if_ready, schedule_after, TaskType};

    const INTERVAL_PROCESSING: Duration = Duration::from_secs(5);

    let task = match pop_if_ready() {
        Some(task) => task,
        None => return,
    };

    match task.task_type {
        TaskType::ProcessLogic => {
            ic_cdk::spawn(async {
                let _guard = match crate::guard::TimerLogicGuard::new() {
                    Some(guard) => guard,
                    None => return,
                };

                let _enqueue_followup_guard = guard((), |_| {
                    schedule_after(INTERVAL_PROCESSING, TaskType::ProcessLogic)
                });

                crate::process_pending_swaps().await;
            });
        }
        TaskType::FetchPrice => {
            ic_cdk::spawn(async {
                const FETCH_RETRY_DELAY_MINUTES: u64 = 10 * 60;
                let call_result = get_exchange_rate().await;
                if let Ok(GetExchangeRateResult::Ok(exchange_rate_result)) = call_result {
                    let icp_price_e8s = convert_to_8_decimals(
                        exchange_rate_result.rate,
                        exchange_rate_result.metadata.decimals,
                    );
                    mutate_state(|s| {
                        s.icp_prices.insert(
                            Timestamp {
                                timestamp_nanos: exchange_rate_result.timestamp,
                            },
                            IcpPrice {
                                rate: icp_price_e8s,
                            },
                        )
                    });
                    // We have a new price entry we should check the
                    // leverage positions that we have.
                    schedule_now(TaskType::CheckLeveragePositions);
                }
                // We fetch data price every 2 minutes
                // Even if the call failed.
                schedule_after(
                    Duration::from_secs(FETCH_RETRY_DELAY_MINUTES),
                    TaskType::FetchPrice,
                );
            });
        }
        TaskType::ProtocolBalanceUpdate => ic_cdk::spawn(async {
            if let Ok(balance) = crate::management::balance_of(ic_cdk::id()).await {
                mutate_state(|s| s.protocol_balance = balance);
                let known_balance = crate::read_state(|s| {
                    s.icp_collateral_amount
                        + s.icp_liqudity_amount
                        + s.icp_leverage_margin_amount
                        + s.liquidity_rewards.values().sum::<u64>()
                });
                debug_assert!(known_balance <= balance);
            }
            schedule_after(Duration::from_secs(1), TaskType::ProtocolBalanceUpdate);
        }),
        TaskType::CheckLeveragePositions => ic_cdk::spawn(async {
            crate::updates::leverage::check_leverage_positions().await;
        }),
        TaskType::CloseLeveragePosition(leverage_position) => ic_cdk::spawn(async {
            let last_icp_price = crate::read_state(|s| s.get_last_icp_price()).unwrap();
            let deposit_block_index = leverage_position.deposit_block_index;
            let owner = leverage_position.owner;
            let now = ic_cdk::api::time();
            let pnl_amount = compute_pnl(&leverage_position.clone(), last_icp_price.rate);
            let amount_to_transfer = if pnl_amount > 0 {
                pnl_amount as u64 + leverage_position.amount
            } else {
                debug_assert!(leverage_position.amount > pnl_amount as u64);
                leverage_position.amount - pnl_amount as u64
            };
            let protocol_fee =
                crate::multiply_e8s(crate::read_state(|s| s.fees.base_fee), amount_to_transfer);
            match crate::transfer_icp(None, owner, amount_to_transfer - protocol_fee).await {
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
                }
                Err(e) => {
                    ic_canister_log::log!(
                        crate::P1,
                        "[task] CloseLeveragePosition failed with {:?}",
                        e
                    );
                    schedule_now(TaskType::CloseLeveragePosition(leverage_position));
                }
            }
        }),
    }
}
