use crate::logs::P1;
use crate::management::transfer_icp;
use crate::state::audit::record_swap_success;
use crate::state::mutate_state;
use crate::state::read_state;
use crate::state::Asset;
use crate::tasks::{schedule_now, TaskType};
use ic_base_types::PrincipalId;
use ic_canister_log::log;
use ic_crypto_sha::Sha256;

pub mod dashboard;
pub mod guard;
pub mod lifecycle;
pub mod logs;
pub mod management;
pub mod metrics;
pub mod state;
pub mod storage;
pub mod tasks;
pub mod timer;
pub mod updates;

pub const E8S_FLOAT: f64 = 100_000_000.0;
pub const E8S: u64 = 100_000_000;
pub const SEC_NANOS: u64 = 1_000_000_000;
pub const ONE_HOUR_NANOS: u64 = 60 * 60 * SEC_NANOS;
pub const EUSD_TRANSFER_FEE: u64 = 1_000_000;

pub const ICP_TRANSFER_FEE: u64 = 10_000;

/// Compute the subaccount of a principal based on a given nonce.
pub fn compute_subaccount(controller: PrincipalId, nonce: u64) -> [u8; 32] {
    const DOMAIN: &[u8] = b"core";
    const DOMAIN_LENGTH: [u8; 1] = [0x04];

    let mut hasher = Sha256::new();
    hasher.write(&DOMAIN_LENGTH);
    hasher.write(DOMAIN);
    hasher.write(controller.as_slice());
    hasher.write(&nonce.to_be_bytes());
    hasher.finish()
}

pub fn multiply_e8s(amount: u64, rate: u64) -> u64 {
    let amount_u128 = amount as u128;
    let rate_u128 = rate as u128;
    let result_u128 = amount_u128 * rate_u128 / 10_u128.pow(8);
    result_u128 as u64
}

pub fn divide_e8s(amount: u64, divisor: u64) -> u64 {
    let amount_u128 = amount as u128;
    let divisor_u128 = divisor as u128;
    let result_u128 = amount_u128 * 10_u128.pow(8) / divisor_u128;
    result_u128 as u64
}

pub async fn process_pending_swaps() {
    let open_swaps = mutate_state(|s| s.open_swaps.clone());
    for (_index, swap) in open_swaps {
        match swap.from {
            Asset::ICP => {
                let amount_to_swap = swap.from_amount - swap.fee;
                let eusd_to_mint = crate::multiply_e8s(amount_to_swap, swap.rate);
                match crate::management::mint_eusd(eusd_to_mint, swap.caller).await {
                    Ok(block_index) => {
                        log!(
                            P1,
                            "[swap]: Success swap from {} {:?} to {:?} ",
                            swap.from_amount,
                            swap.from,
                            swap.to,
                        );
                        mutate_state(|s| {
                            record_swap_success(s, swap.from_block_index, block_index);
                        });
                    }
                    Err(e) => {
                        log!(
                            P1,
                            "[swap]: failed to swap from {:?} to {:?}, error: {:?}",
                            swap.from,
                            swap.to,
                            e
                        );
                    }
                }
            }
            Asset::EUSD => {
                let amount_to_swap = swap.from_amount - swap.fee;
                let icp_to_transfer = divide_e8s(amount_to_swap, swap.rate);
                match crate::management::transfer_icp(None, swap.caller, icp_to_transfer).await {
                    Ok(block_index) => {
                        log!(
                            P1,
                            "[swap]: Success swap from {} {:?} to {} {:?} ",
                            swap.from_amount,
                            swap.from,
                            icp_to_transfer,
                            swap.to,
                        );
                        mutate_state(|s| {
                            record_swap_success(s, swap.from_block_index, block_index);
                        });
                    }
                    Err(_) => {
                        log!(
                            P1,
                            "[swap]: failed to swap from {:?} to {:?}",
                            swap.from,
                            swap.to
                        );
                    }
                }
            }
        }
    }
    schedule_now(TaskType::ProcessLogic);
}

#[test]
fn test_multiply_e8s() {
    let amount: u64 = 150_000_001; // 1.5 ICP
    let rate: u64 = 520_000_000; // 5.2 $

    let multiplication_result = multiply_e8s(amount, rate);
    assert_eq!(multiplication_result, 780_000_005);
}

#[test]
fn test_divide_e8s() {
    let amount: u64 = 780_000_005; // 7.8 $
    let divisor: u64 = 520_000_000; // 5.2 $

    let division_result = divide_e8s(amount, divisor);
    assert_eq!(division_result, 150_000_000);
}

#[cfg(test)]
mod tests {
    use crate::compute_subaccount;
    use ic_base_types::PrincipalId;
    use std::str::FromStr;

    #[test]
    fn test_compute_subaccount() {
        let pid: PrincipalId = PrincipalId::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let expected: [u8; 32] = [
            171, 1, 10, 73, 118, 128, 21, 217, 95, 129, 152, 106, 123, 123, 118, 52, 150, 160, 45,
            204, 28, 121, 247, 95, 232, 248, 125, 84, 161, 155, 192, 154,
        ];
        assert_eq!(expected, compute_subaccount(pid, 0));
    }
}
