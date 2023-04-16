use candid::{Decode, Encode, Principal};
use core_canister::state::{ProtocolStatus, UserData};
use core_canister::updates::leverage::{LeveragePositionError, OpenLeveragePositionArg};
use core_canister::updates::liquidity::LiquidityError;
use core_canister::updates::swap::{SwapArg, SwapError};
use ic_base_types::PrincipalId;
use ic_canisters_http_types::{HttpRequest, HttpResponse};
use ic_state_machine_tests::{CanisterId, StateMachine};
use icrc_ledger_types::icrc1::account::Account;
use std::collections::BTreeMap;

pub fn send_swap(
    env: &StateMachine,
    coreid: CanisterId,
    from: Principal,
    arg: &SwapArg,
) -> Result<u64, SwapError> {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            coreid,
            "swap",
            Encode!(arg)
            .unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        Result<u64, SwapError>
    )
    .expect("failed to decode transfer response")
}

pub fn send_open_leverage(
    env: &StateMachine,
    core_id: CanisterId,
    from: Principal,
    arg: &OpenLeveragePositionArg,
) -> Result<u64, LeveragePositionError> {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            core_id,
            "open_leverage_position",
            Encode!(arg)
            .unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        Result<u64, LeveragePositionError>
    )
    .expect("failed to decode transfer response")
}

pub fn send_close_leverage(
    env: &StateMachine,
    core_id: CanisterId,
    from: Principal,
    arg: &u64,
) -> Result<u64, LeveragePositionError> {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            core_id,
            "close_leverage_position",
            Encode!(arg)
            .unwrap()
        )
        .expect("failed to close leverage position")
        .bytes(),
        Result<u64, LeveragePositionError>
    )
    .expect("failed to decode transfer response")
}

pub fn send_add_liquidity(
    env: &StateMachine,
    core_id: CanisterId,
    from: Principal,
    arg: &u64,
) -> Result<u64, LiquidityError> {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            core_id,
            "add_liquidity",
            Encode!(arg)
            .unwrap()
        )
        .expect("failed to add liquidity")
        .bytes(),
        Result<u64, LiquidityError>
    )
    .expect("failed to decode transfer response")
}

pub fn send_remove_liquidity(
    env: &StateMachine,
    core_id: CanisterId,
    from: Principal,
    arg: &u64,
) -> Result<u64, LiquidityError> {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            core_id,
            "remove_liquidity",
            Encode!(arg)
            .unwrap()
        )
        .expect("failed to add liquidity")
        .bytes(),
        Result<u64, LiquidityError>
    )
    .expect("failed to decode transfer response")
}

pub fn get_deposit_account(env: &StateMachine, coreid: CanisterId, from: Principal) -> Account {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            coreid,
            "get_deposit_account",
            Encode!().unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        Account
    )
    .expect("failed to decode transfer response")
}

pub fn get_protocol_status(env: &StateMachine, core_id: CanisterId) -> ProtocolStatus {
    Decode!(
        &env.query(core_id, "get_protocol_status", Encode!().unwrap())
            .expect("failed to query protocol status")
            .bytes(),
        ProtocolStatus
    )
    .expect("failed to decode get_protocol_status response")
}

pub fn get_user_data(env: &StateMachine, core_id: CanisterId, target: &Principal) -> UserData {
    Decode!(
        &env.query(core_id, "get_user_data", Encode!(target).unwrap())
            .expect("failed to query user data")
            .bytes(),
        UserData
    )
    .expect("failed to decode get_user_data response")
}

pub fn get_logs(env: &StateMachine, core_id: CanisterId) -> Vec<String> {
    Decode!(
        &env.query(core_id, "get_logs", Encode!().unwrap())
            .expect("failed to query logs")
            .bytes(),
        Vec<String>
    )
    .expect("failed to decode get_logs response")
}

pub fn self_check(env: &StateMachine, core_id: CanisterId) -> Result<(), String> {
    Decode!(
        &env.query(core_id, "self_check", Encode!().unwrap())
            .expect("failed to transfer funds")
            .bytes(),
            Result<(), String>
    )
    .expect("failed to decode transfer response")
}

pub fn get_metrics(env: &StateMachine, core_id: CanisterId) -> BTreeMap<String, u64> {
    let raw_metrics = Decode!(
        &env.query(
            core_id,
            "http_request",
            Encode!(&HttpRequest {
                method: "GET".into(),
                url: "/metrics".into(),
                headers: vec![],
                body: Default::default(),
            })
            .unwrap(),
        )
        .expect("failed to transfer funds")
        .bytes(),
        HttpResponse
    )
    .expect("failed to decode transfer response");
    let binding = raw_metrics.body.into_vec();
    let metrics_body = std::str::from_utf8(&binding).unwrap();
    let metrics_vec: Vec<String> = metrics_body
        .trim()
        .split('\n')
        .filter(|s| !s.trim().starts_with('#'))
        .map(|s| s.to_string())
        .collect();
    let mut map = BTreeMap::new();
    for s in metrics_vec {
        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() == 3 {
            map.insert(
                parts[0].to_string(),
                parts[1].to_string().parse::<f64>().unwrap() as u64,
            );
        }
    }
    map
}

pub fn get_known_protocol_balance(env: &StateMachine, core_id: CanisterId) -> u64 {
    let metrics = get_metrics(env, core_id);
    dbg!(metrics.clone());
    let collateral_amount = metrics.get(&"core_collateral_amount".to_string()).unwrap();
    let core_liquidity_amount = metrics.get(&"core_liquidity_amount".to_string()).unwrap();
    let core_leverage_margin_amount = metrics
        .get(&"core_leverage_margin_amount".to_string())
        .unwrap();
    let core_rewards_amount = metrics
        .get(&"core_total_claimable_rewards".to_string())
        .unwrap();

    collateral_amount + core_liquidity_amount + core_leverage_margin_amount + core_rewards_amount
}
