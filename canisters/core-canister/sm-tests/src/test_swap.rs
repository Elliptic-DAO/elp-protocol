use crate::calls::core_canister::send_swap;
use crate::calls::{
    ledger::{get_balance_of, send_transfer},
    xrc_canister::{assert_xrc_is_running, upgrade_icp_price},
};
use crate::{ONE_E8S, TEN_E8S};
use assert_matches::assert_matches;
use core_canister::state::Asset;
use core_canister::updates::swap::SwapArg;
use ic_base_types::PrincipalId;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use num_traits::ToPrimitive;
use std::time::Duration;

pub fn test_swap(core_canister_wasm: Vec<u8>, xrc_wasm: Vec<u8>, icrc1_ledger_wasm: Vec<u8>) {
    let mut initial_balances: Vec<(Account, u64)> = vec![];
    let number_of_users = 10;
    let users = crate::get_users(number_of_users);
    let icp_minter = PrincipalId::new(0, [0u8; 29]).0;
    for user in &users {
        initial_balances.push((
            Account {
                owner: *user,
                subaccount: None,
            },
            crate::ONE_THOUSAND_E8S,
        ));
    }
    let initial_icp_rate: u64 = 500_000_000; // 5$
    let (env, canister_ids) = crate::setup::setup(
        xrc_wasm.clone(),
        icrc1_ledger_wasm,
        core_canister_wasm,
        initial_balances,
        initial_icp_rate,
    );
    let users_deposit_accounts =
        crate::get_user_deposit_account(&env, number_of_users, canister_ids.core_id, users.clone());

    env.advance_time(Duration::from_secs(60));
    env.run_until_completion(1000);

    // Send ICP to subaccount
    let transfer_arg = TransferArg {
        from_subaccount: None,
        to: users_deposit_accounts[0],
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
        amount: ONE_E8S,
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
    // 498_750_000
    let balance_first_swap: u64 = 498_750_000;
    assert_eq!(balance_of_result, balance_first_swap);

    let new_price: u64 = 1_000_000_000; // 10$
    upgrade_icp_price(&env, canister_ids.xrc_id, xrc_wasm.clone(), new_price);
    let swap_arg = SwapArg {
        from_asset: Asset::ICP,
        to_asset: Asset::EUSD,
        amount: ONE_E8S,
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
    let balance_second_swap = 997_500_000;
    assert_eq!(balance_of_result, balance_first_swap + balance_second_swap);

    upgrade_icp_price(&env, canister_ids.xrc_id, xrc_wasm, initial_icp_rate);

    for k in 1..10 {
        let transfer_arg = TransferArg {
            from_subaccount: None,
            to: users_deposit_accounts[k],
            fee: None,
            created_at_time: None,
            memo: None,
            amount: TEN_E8S.into(),
        };
        let transfer_result =
            send_transfer(&env, canister_ids.icp_ledger_id, users[k], &transfer_arg);
        assert_matches!(transfer_result, Ok(_));
        let balance_of_result_icp =
            get_balance_of(&env, canister_ids.icp_ledger_id, &users_deposit_accounts[k]);
        dbg!(k, balance_of_result_icp);
        let swap_result = send_swap(&env, canister_ids.core_id, users[k], &swap_arg);
        assert_matches!(swap_result, Ok(_));
        let balance_of_result = get_balance_of(
            &env,
            canister_ids.eusd_ledger_id,
            &Account {
                owner: users[k],
                subaccount: None,
            },
        );
        assert_eq!(balance_of_result, 0);
    }

    env.advance_time(Duration::from_secs(60));
    env.tick();

    for k in 1..10 {
        let balance_of_result = get_balance_of(
            &env,
            canister_ids.eusd_ledger_id,
            &Account {
                owner: users[k],
                subaccount: None,
            },
        );
        let balance_of_result_icp =
            get_balance_of(&env, canister_ids.icp_ledger_id, &users_deposit_accounts[k]);
        dbg!(k, balance_of_result_icp);
        assert_eq!(balance_of_result, balance_first_swap);
    }

    for k in 1..10 {
        env.tick();
        let balance_of_result_icp_before_swap = get_balance_of(
            &env,
            canister_ids.icp_ledger_id,
            &Account {
                owner: users[k],
                subaccount: None,
            },
        )
        .0
        .to_u64()
        .expect("Error while converting nat to u64");
        let amount_to_swap = balance_first_swap - core_canister::EUSD_TRANSFER_FEE;
        let transfer_arg = TransferArg {
            from_subaccount: None,
            to: users_deposit_accounts[k],
            fee: None,
            created_at_time: None,
            memo: None,
            amount: amount_to_swap.into(),
        };
        let transfer_result =
            send_transfer(&env, canister_ids.eusd_ledger_id, users[k], &transfer_arg);
        assert_matches!(transfer_result, Ok(_));
        let swap_arg = SwapArg {
            from_asset: Asset::EUSD,
            to_asset: Asset::ICP,
            amount: amount_to_swap,
        };
        let swap_result = send_swap(&env, canister_ids.core_id, users[k], &swap_arg);
        assert_matches!(swap_result, Ok(_));
        let balance_of_subaccount = get_balance_of(
            &env,
            canister_ids.eusd_ledger_id,
            &users_deposit_accounts[k],
        )
        .0
        .to_u64()
        .expect("Error while converting nat to u64");
        assert_eq!(balance_of_subaccount, 0);
        env.advance_time(Duration::from_secs(60));
        env.tick();
        let balance_of_result_icp_after_swap = get_balance_of(
            &env,
            canister_ids.icp_ledger_id,
            &Account {
                owner: users[k],
                subaccount: None,
            },
        )
        .0
        .to_u64()
        .expect("Error while converting nat to u64");
        let amount_minted = balance_of_result_icp_after_swap - balance_of_result_icp_before_swap;
        // TODO check that the minted amount correspond to expectation.

        dbg!(
            k,
            balance_of_result_icp_before_swap,
            balance_of_result_icp_after_swap,
            amount_minted
        );
    }
}
