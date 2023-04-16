use crate::calls::core_canister::{
    get_deposit_account, get_known_protocol_balance, get_metrics, get_protocol_status,
    get_user_data, send_add_liquidity, send_close_leverage, send_open_leverage,
    send_remove_liquidity, send_swap,
};
use crate::calls::{
    ledger::{get_balance_of, send_transfer},
    xrc_canister::assert_xrc_is_running,
};
use crate::setup::LedgerArgument;
use assert_matches::assert_matches;
use candid::{Encode, Principal};
use core_canister::state::Asset;
use core_canister::updates::leverage::{LeveragePositionError, OpenLeveragePositionArg};
use core_canister::updates::liquidity::LiquidityError;
use core_canister::updates::swap::SwapArg;
use ic_base_types::CanisterId;
use ic_base_types::PrincipalId;
use ic_state_machine_tests::StateMachine;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use std::time::Duration;

pub mod calls;
pub mod setup;
pub mod test_swap;

const ONE_THOUSAND_E8S: u64 = 10_000_000_000;
const TEN_E8S: u64 = 1_000_000_000;
const FIVE_E8S: u64 = 500_000_000;
const ONE_E8S: u64 = 100_000_000;
const ICP_TRANSFER_FEE: u64 = 10_000;

fn assert_balances_consistency(env: &StateMachine, core_id: CanisterId, icp_ledger_id: CanisterId) {
    let kown_balance = get_known_protocol_balance(env, core_id);
    let protocol_balance = get_balance_of(
        env,
        icp_ledger_id,
        &Account {
            owner: core_id.into(),
            subaccount: None,
        },
    );
    assert_eq!(kown_balance, protocol_balance);
}

fn get_users(amount: u8) -> Vec<Principal> {
    let mut users = vec![];
    for k in 1..(amount + 1) {
        users.push(PrincipalId::new(1, [k; 29]).0);
    }
    users
}

fn get_user_deposit_account(
    env: &StateMachine,
    amount: u8,
    core_id: CanisterId,
    users: Vec<Principal>,
) -> Vec<Account> {
    let mut deposit_accounts: Vec<Account> = vec![];
    for k in 0..amount {
        let deposit_account = get_deposit_account(env, core_id, users[k as usize]);
        deposit_accounts.push(deposit_account);
    }
    deposit_accounts
}

pub fn test_core(core_canister_wasm: Vec<u8>, xrc_wasm: Vec<u8>, icrc1_ledger_wasm: Vec<u8>) {
    let mut initial_balances: Vec<(Account, u64)> = vec![];

    let users = get_users(10);
    let icp_minter = PrincipalId::new(0, [0u8; 29]).0;
    for user in &users {
        initial_balances.push((
            Account {
                owner: *user,
                subaccount: None,
            },
            ONE_THOUSAND_E8S,
        ));
    }
    let initial_icp_rate: u64 = 1_000_000_000;
    let (env, canister_ids) = crate::setup::setup(
        xrc_wasm,
        icrc1_ledger_wasm,
        core_canister_wasm.clone(),
        initial_balances,
        initial_icp_rate,
    );

    env.advance_time(Duration::from_secs(60));
    env.run_until_completion(1000);

    dbg!("assert_balances_consistency 1");
    assert_balances_consistency(&env, canister_ids.core_id, canister_ids.icp_ledger_id);

    // Send ICP to subaccount
    let deposit_account_user_0 = get_deposit_account(&env, canister_ids.core_id, users[0]);
    let transfer_arg = TransferArg {
        from_subaccount: None,
        to: deposit_account_user_0,
        fee: None,
        created_at_time: None,
        memo: None,
        amount: TEN_E8S.into(),
    };
    let transfer_result = send_transfer(&env, canister_ids.icp_ledger_id, users[0], &transfer_arg);
    assert_matches!(transfer_result, Ok(_));

    assert_xrc_is_running(&env, canister_ids.xrc_id, icp_minter);

    let swap_arg = SwapArg {
        from_asset: Asset::ICP,
        to_asset: Asset::EUSD,
        amount: TEN_E8S - ICP_TRANSFER_FEE,
    };
    let swap_result = send_swap(&env, canister_ids.core_id, users[0], &swap_arg);
    assert_matches!(swap_result, Ok(_));

    env.advance_time(Duration::from_secs(60));
    env.tick();

    let balance_of_result = get_balance_of(
        &env,
        canister_ids.eusd_ledger_id,
        &Account {
            owner: users[0],
            subaccount: None,
        },
    );
    assert_eq!(balance_of_result, 9974900250_u64);

    let deposit_account_user_1 = get_deposit_account(&env, canister_ids.core_id, users[1]);
    let transfer_arg = TransferArg {
        from_subaccount: None,
        to: deposit_account_user_1,
        fee: None,
        created_at_time: None,
        memo: None,
        amount: FIVE_E8S.into(),
    };
    let transfer_result = send_transfer(&env, canister_ids.icp_ledger_id, users[1], &transfer_arg);
    assert_matches!(transfer_result, Ok(_));

    let protocol_status = get_protocol_status(&env, canister_ids.core_id);
    assert_eq!(protocol_status.collateral_ratio, 100_000_000);

    let open_leverage_result = send_open_leverage(
        &env,
        canister_ids.core_id,
        users[1],
        &OpenLeveragePositionArg {
            amount: TEN_E8S,
            take_profit: 1_500_000_000,
            covered_amount: TEN_E8S,
        },
    );
    assert_matches!(
        open_leverage_result,
        Err(LeveragePositionError::NotEnoughFundsToCover)
    );
    let open_leverage_result = send_open_leverage(
        &env,
        canister_ids.core_id,
        users[1],
        &OpenLeveragePositionArg {
            amount: FIVE_E8S - ICP_TRANSFER_FEE,
            take_profit: 1_500_000_000,
            covered_amount: FIVE_E8S,
        },
    );
    assert_matches!(open_leverage_result, Ok(_));

    dbg!("assert_balances_consistency 2");
    assert_balances_consistency(&env, canister_ids.core_id, canister_ids.icp_ledger_id);

    let user_data = get_user_data(&env, canister_ids.core_id, &users[1]);
    assert_matches!(user_data.leverage_positions, Some(_));
    let metrics = get_metrics(&env, canister_ids.core_id);
    dbg!(metrics);
    let protocol_status = get_protocol_status(&env, canister_ids.core_id);
    // dbg!(protocol_status);
    // (500_000_000 - 10_000) * 0.9975
    // (1_000_000_000 - 10_000) * 0.9975
    // (997490025.0 + 498740025.0) * 10
    // 14962300500 / 9974900250 = 149999499
    assert_eq!(protocol_status.collateral_ratio, 149_999_499);

    let deposit_account_user_2 = get_deposit_account(&env, canister_ids.core_id, users[2]);
    let transfer_arg = TransferArg {
        from_subaccount: None,
        to: deposit_account_user_2,
        fee: None,
        created_at_time: None,
        memo: None,
        amount: TEN_E8S.into(),
    };
    let transfer_result = send_transfer(&env, canister_ids.icp_ledger_id, users[2], &transfer_arg);
    assert_matches!(transfer_result, Ok(_));

    let add_liquidity_result = send_add_liquidity(&env, canister_ids.core_id, users[2], &FIVE_E8S);
    assert_matches!(add_liquidity_result, Ok(_));

    dbg!("assert balances 3");
    assert_balances_consistency(&env, canister_ids.core_id, canister_ids.icp_ledger_id);

    let user_data = get_user_data(&env, canister_ids.core_id, &users[2]);
    assert_eq!(user_data.claimable_liquidity_rewards, 4999950);
    assert_eq!(user_data.liquidity_provided, 498_750_000);

    let remove_liquidity_result =
        send_remove_liquidity(&env, canister_ids.core_id, users[2], &498_750_001);
    assert_matches!(
        remove_liquidity_result,
        Err(LiquidityError::NotEnoughLiquidity(498750000))
    );
    let remove_liquidity_result =
        send_remove_liquidity(&env, canister_ids.core_id, users[2], &498_750_000);
    assert_matches!(remove_liquidity_result, Ok(_));

    dbg!("assert balances 4");
    assert_balances_consistency(&env, canister_ids.core_id, canister_ids.icp_ledger_id);

    let user2_data = get_user_data(&env, canister_ids.core_id, &users[2]);
    assert_eq!(user2_data.liquidity_provided, 0);
    // assert_eq!(user2_data.claimable_liquidity_rewards, 4999950);

    let protocol_status_before_upgrade = get_protocol_status(&env, canister_ids.core_id);
    let upgrade_args = LedgerArgument::Upgrade(None);
    env.upgrade_canister(
        canister_ids.core_id,
        core_canister_wasm,
        Encode!(&upgrade_args).unwrap(),
    )
    .expect("failed to upgrade the archive canister");
    let protocol_status_after_upgrade = get_protocol_status(&env, canister_ids.core_id);
    assert_eq!(
        protocol_status_before_upgrade,
        protocol_status_after_upgrade
    );
    let user2_data_after_upgrade = get_user_data(&env, canister_ids.core_id, &users[2]);
    assert_eq!(user2_data, user2_data_after_upgrade);
    let remove_liquidity_result =
        send_remove_liquidity(&env, canister_ids.core_id, users[2], &498750000);
    assert_matches!(
        remove_liquidity_result,
        Err(LiquidityError::NotEnoughLiquidity(0))
    );
    let open_leverage_block_index = open_leverage_result.unwrap();
    let close_leverage_result = send_close_leverage(
        &env,
        canister_ids.core_id,
        users[1],
        &open_leverage_block_index,
    );
    assert_matches!(
        close_leverage_result,
        Err(LeveragePositionError::TooEarlyToClose)
    );

    env.advance_time(Duration::from_secs(60 * 60 * 2)); // 2 hours later
    let close_leverage_result = send_close_leverage(
        &env,
        canister_ids.core_id,
        users[1],
        &open_leverage_block_index,
    );
    assert_matches!(close_leverage_result, Ok(_));

    let protocol_balance = get_balance_of(
        &env,
        canister_ids.icp_ledger_id,
        &Account {
            owner: canister_ids.core_id.into(),
            subaccount: None,
        },
    );

    let metrics = get_metrics(&env, canister_ids.core_id);
    dbg!(metrics.clone());
    let collateral_amount = metrics.get(&"core_collateral_amount".to_string()).unwrap();
    let core_liquidity_amount = metrics.get(&"core_liquidity_amount".to_string()).unwrap();
    let core_leverage_margin_amount = metrics
        .get(&"core_leverage_margin_amount".to_string())
        .unwrap();
    let core_rewards_amount = metrics
        .get(&"core_total_claimable_rewards".to_string())
        .unwrap();
    let balances_sum = collateral_amount
        + core_liquidity_amount
        + core_leverage_margin_amount
        + core_rewards_amount;
    dbg!(balances_sum, protocol_balance.clone());
    assert!(
        collateral_amount
            + core_liquidity_amount
            + core_leverage_margin_amount
            + core_rewards_amount
            == protocol_balance
    );
}
