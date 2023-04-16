use assert_matches::assert_matches;
use candid::{Decode, Encode, Principal};
use ic_base_types::PrincipalId;
use ic_state_machine_tests::{CanisterId, StateMachine};
use ic_xrc_types::{Asset as XrcAsset, AssetClass};
use ic_xrc_types::{GetExchangeRateRequest, GetExchangeRateResult};
use xrc_mock::{ExchangeRate as ExchangeRateMock, Response, XrcMockInitPayload};

pub fn send_fetch_icp_price(
    env: &StateMachine,
    xrc_id: CanisterId,
    from: Principal,
    arg: &GetExchangeRateRequest,
) -> GetExchangeRateResult {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            xrc_id,
            "get_exchange_rate",
            Encode!(arg).unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        GetExchangeRateResult
    )
    .expect("failed to decode transfer response")
}

pub fn upgrade_icp_price(env: &StateMachine, xrc_id: CanisterId, xrc_wasm: Vec<u8>, rate: u64) {
    let xrc_args = XrcMockInitPayload {
        response: Response::ExchangeRate(ExchangeRateMock {
            base_asset: Some(XrcAsset {
                symbol: "ICP".into(),
                class: AssetClass::Cryptocurrency,
            }),
            quote_asset: Some(XrcAsset {
                symbol: "USD".into(),
                class: AssetClass::FiatCurrency,
            }),
            metadata: None,
            rate,
        }),
    };
    env.reinstall_canister(xrc_id, xrc_wasm, Encode!(&xrc_args).unwrap())
        .expect("failed to upgrade the archive canister");
}

pub fn assert_xrc_is_running(env: &StateMachine, xrc_id: CanisterId, caller: Principal) {
    let exchange_rate_arg = GetExchangeRateRequest {
        base_asset: XrcAsset {
            symbol: "ICP".into(),
            class: AssetClass::Cryptocurrency,
        },
        quote_asset: XrcAsset {
            symbol: "USD".into(),
            class: AssetClass::FiatCurrency,
        },
        timestamp: Some(0),
    };
    let fetch_price_result = send_fetch_icp_price(env, xrc_id, caller, &exchange_rate_arg);
    assert_matches!(fetch_price_result, Ok(_));
}
