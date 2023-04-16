use crate::state::{read_state, Mode};
use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::Nat;
use ic_base_types::PrincipalId;
use ic_cdk::export::Principal;
use ic_icrc1_client_cdk::{CdkRuntime, ICRC1Client};
use ic_xrc_types::{Asset, AssetClass, GetExchangeRateRequest, GetExchangeRateResult};
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use icrc_ledger_types::icrc1::transfer::TransferArg;
use icrc_ledger_types::icrc1::transfer::TransferError;

const XRC_MARGIN_SEC: u64 = 5 * 60;
// The payment required for querying the XRC canister.
const XRC_CALL_COST_CYCLES: u64 = 10_000_000_000;

/// Query the XRC canister to retrieve the last ICP/USD price.
pub async fn get_exchange_rate() -> Result<GetExchangeRateResult, String> {
    let icp = Asset {
        symbol: "ICP".to_string(),
        class: AssetClass::Cryptocurrency,
    };
    let usd = Asset {
        symbol: "USD".to_string(),
        class: AssetClass::FiatCurrency,
    };

    // Take few minutes back to be sure to have data.
    let timestamp_sec = ic_cdk::api::time() / crate::SEC_NANOS - XRC_MARGIN_SEC;

    // Retrieve last ICP/USD value.
    let args = GetExchangeRateRequest {
        base_asset: icp,
        quote_asset: usd,
        timestamp: Some(timestamp_sec),
    };

    let xrc_principal = read_state(|s| s.xrc_principal);

    ic_cdk::println!("Calling XRC canister ({})", xrc_principal);
    let res_xrc: Result<(GetExchangeRateResult,), (i32, String)> =
        match read_state(|s| s.mode.clone()) {
            Mode::NoHttpOutCalls => call(xrc_principal, "get_exchange_rate", (args,)).await,
            _ => {
                call_with_payment(
                    xrc_principal,
                    "get_exchange_rate",
                    (args,),
                    XRC_CALL_COST_CYCLES,
                )
                .await
            }
        };
    match res_xrc {
        Ok((xr,)) => Ok(xr),
        Err((code, msg)) => Err(format!(
            "Error while calling XRC canister ({}): {:?}",
            code, msg
        )),
    }
}

pub async fn mint_eusd(amount: u64, to: Principal) -> Result<u64, TransferError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: crate::state::read_state(|s| s.eusd_ledger_principal),
    };
    let block_index = client
        .transfer(TransferArg {
            from_subaccount: None,
            to: Account {
                owner: to,
                subaccount: None,
            },
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        })
        .await
        .map_err(|e| TransferError::GenericError {
            error_code: (Nat::from(e.0)),
            message: (e.1),
        })??;
    Ok(block_index)
}

pub async fn burn_eusd(user: Principal, amount: u64) -> Result<u64, TransferError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: read_state(|s| s.eusd_ledger_principal),
    };
    let core_id = ic_cdk::id();
    let from_subaccount = crate::compute_subaccount(PrincipalId(user), 0);
    let block_index = client
        .transfer(TransferArg {
            from_subaccount: Some(from_subaccount),
            to: Account {
                owner: core_id,
                subaccount: None,
            },
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        })
        .await
        .map_err(|e| TransferError::GenericError {
            error_code: (Nat::from(e.0)),
            message: (e.1),
        })??;
    Ok(block_index)
}

pub async fn transfer_icp(
    from_subaccount: Option<Subaccount>,
    to: Principal,
    amount: u64,
) -> Result<u64, TransferError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: read_state(|s| s.icp_ledger_principal),
    };
    let block_index = client
        .transfer(TransferArg {
            from_subaccount,
            to: Account {
                owner: to,
                subaccount: None,
            },
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        })
        .await
        .map_err(|e| TransferError::GenericError {
            error_code: (Nat::from(e.0)),
            message: (e.1),
        })??;
    Ok(block_index)
}

pub async fn balance_of(owner: Principal) -> Result<u64, TransferError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: read_state(|s| s.icp_ledger_principal),
    };
    let balance = client
        .balance_of(Account {
            owner,
            subaccount: None,
        })
        .await
        .map_err(|e| TransferError::GenericError {
            error_code: (Nat::from(e.0)),
            message: (e.1),
        })?;
    Ok(balance)
}

async fn call_with_payment<In, Out>(
    id: Principal,
    method: &str,
    args: In,
    cycles: u64,
) -> Result<Out, (i32, String)>
where
    In: ArgumentEncoder + Send,
    Out: for<'a> ArgumentDecoder<'a>,
{
    ic_cdk::api::call::call_with_payment(id, method, args, cycles)
        .await
        .map_err(|(code, msg)| (code as i32, msg))
}

async fn call<In, Out>(id: Principal, method: &str, args: In) -> Result<Out, (i32, String)>
where
    In: ArgumentEncoder + Send,
    Out: for<'a> ArgumentDecoder<'a>,
{
    ic_cdk::call(id, method, args)
        .await
        .map_err(|(code, msg)| (code as i32, msg))
}
