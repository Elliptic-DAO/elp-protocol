use crate::liquidity::LiquidityError;
use candid::candid_method;
use core_canister::dashboard::build_dashboard;
use core_canister::lifecycle::{init::CoreArgs, upgrade::UpgradeArgs};
use core_canister::logs::P1;
use core_canister::metrics::encode_metrics;
use core_canister::state::{
    eventlog::{Event, GetEventsArg},
    read_state, Asset, IcpPrice, ProtocolStatus, UserData,
};
use core_canister::tasks::schedule_now;
use core_canister::tasks::TaskType;
use core_canister::updates::leverage::{LeveragePositionError, OpenLeveragePositionArg};
use core_canister::updates::liquidity;
use core_canister::updates::swap::{SwapArg, SwapError};
use ic_canister_log::export;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk_macros::{init, post_upgrade, query, update};
use icrc_ledger_types::icrc1::account::Account;

fn main() {}

#[cfg(feature = "self_check")]
fn ok_or_die(result: Result<(), String>) {
    if let Err(msg) = result {
        ic_cdk::println!("{}", msg);
        ic_cdk::trap(&msg);
    }
}

/// Checks that Elliptic Core Canister state is internally consistent.
#[cfg(feature = "self_check")]
fn check_invariants() -> Result<(), String> {
    use core_canister::state::eventlog::replay;

    read_state(|s| {
        s.check_invariants()?;

        let events: Vec<_> = core_canister::storage::events().collect();
        let recovered_state = replay(events.clone().into_iter())
            .unwrap_or_else(|e| panic!("failed to replay log {:?}: {:?}", events, e));

        recovered_state.check_invariants()?;

        // A running timer can temporarily violate invariants.
        if !s.is_timer_running {
            s.check_semantically_eq(&recovered_state)?;
        }

        Ok(())
    })
}

fn check_postcondition<T>(t: T) -> T {
    #[cfg(feature = "self_check")]
    ok_or_die(check_invariants());
    t
}

#[init]
fn init(args: CoreArgs) {
    match args {
        CoreArgs::Init(init_args) => {
            core_canister::storage::record_event(&Event::Init(init_args.clone()));
            core_canister::lifecycle::init::init(init_args);
        }
        CoreArgs::Upgrade(_) => panic!("Expected Init args got Upgrade args."),
    }

    schedule_now(TaskType::FetchPrice);
    // schedule_now(TaskType::ProtocolBalanceUpdate);
    schedule_now(TaskType::CheckLeveragePositions);
}

#[post_upgrade]
fn post_upgrade(core_arg: Option<CoreArgs>) {
    let mut upgrade_args: Option<UpgradeArgs> = None;
    if let Some(core_arg) = core_arg {
        upgrade_args = match core_arg {
            CoreArgs::Upgrade(core_arg) => core_arg,
            CoreArgs::Init(_) => panic!("expected Option<UpgradeArgs> got InitArgs."),
        };
    }
    core_canister::lifecycle::upgrade::post_upgrade(upgrade_args);

    schedule_now(TaskType::FetchPrice);
    // schedule_now(TaskType::ProtocolBalanceUpdate);
    schedule_now(TaskType::CheckLeveragePositions);
}

#[candid_method(update)]
#[update]
async fn get_deposit_account() -> Account {
    Account {
        owner: ic_cdk::id(),
        subaccount: Some(core_canister::compute_subaccount(
            ic_cdk::caller().into(),
            0,
        )),
    }
}

#[candid_method(update)]
#[update]
async fn open_leverage_position(
    arg: OpenLeveragePositionArg,
) -> Result<u64, LeveragePositionError> {
    check_postcondition(core_canister::updates::leverage::open_leverage_position(arg).await)
}

#[candid_method(update)]
#[update]
async fn close_leverage_position(position_block_index: u64) -> Result<u64, LeveragePositionError> {
    check_postcondition(
        core_canister::updates::leverage::close_leverage_position(position_block_index).await,
    )
}

#[candid_method(update)]
#[update]
async fn swap(swap_arg: SwapArg) -> Result<u64, SwapError> {
    match swap_arg.from_asset {
        Asset::ICP => check_postcondition(
            core_canister::updates::swap::convert_icp_to_eusd(swap_arg.amount).await,
        ),
        Asset::EUSD => check_postcondition(
            core_canister::updates::swap::convert_eusd_to_icp(swap_arg.amount).await,
        ),
    }
}

#[candid_method(update)]
#[update]
async fn add_liquidity(amount: u64) -> Result<u64, LiquidityError> {
    check_postcondition(core_canister::updates::liquidity::add_liquidity(amount).await)
}

#[candid_method(update)]
#[update]
async fn remove_liquidity(amount: u64) -> Result<u64, LiquidityError> {
    check_postcondition(
        core_canister::updates::liquidity::remove_liquidity(amount, ic_cdk::caller()).await,
    )
}

#[candid_method(update)]
#[update]
async fn claim_liquidity_rewards() -> Result<u64, LiquidityError> {
    check_postcondition(core_canister::updates::liquidity::claim_liquidity_rewards().await)
}

#[candid_method(query)]
#[query]
fn get_user_data(principal: candid::Principal) -> UserData {
    read_state(|s| UserData {
        claimable_liquidity_rewards: s.liquidity_rewards.get(&principal).cloned().unwrap_or(0),
        liquidity_provided: *s.liquidity_provided.get(&principal).unwrap_or(&0),
        leverage_positions: s.get_leverage_position_of(principal),
    })
}

#[candid_method(query)]
#[query]
fn get_protocol_status() -> ProtocolStatus {
    read_state(|s| ProtocolStatus {
        collateral_ratio: s.get_collateral_ratio(),
        coverered_ratio: s.get_coverered_ratio(),
        icp_price: s.get_last_icp_price().unwrap_or(IcpPrice { rate: 0 }).rate,
        tvl: s.get_tvl(),
        coverable_amount: s.get_leverage_coverable_amount(),
    })
}

#[candid_method(query)]
#[query]
fn get_events(args: GetEventsArg) -> Vec<Event> {
    const MAX_EVENTS_PER_QUERY: usize = 2000;

    core_canister::storage::events()
        .skip(args.start as usize)
        .take(MAX_EVENTS_PER_QUERY.min(args.length as usize))
        .collect()
}

#[candid_method(query)]
#[query]
fn get_logs() -> Vec<String> {
    let entries = export(&P1);
    let mut displayed_entries: Vec<String> = vec![];
    for entry in entries {
        displayed_entries.push(format!(
            "{} {} {} {}",
            entry.timestamp, entry.file, entry.line, entry.message
        ));
    }
    displayed_entries
}

#[export_name = "canister_global_timer"]
fn timer() {
    #[cfg(feature = "self_check")]
    ok_or_die(check_invariants());
    core_canister::timer::timer();
}

#[candid_method(query)]
#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    if req.path() == "/metrics" {
        let mut writer =
            ic_metrics_encoder::MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);
        match encode_metrics(&mut writer) {
            Ok(()) => HttpResponseBuilder::ok()
                .header("Content-Type", "text/plain; version=0.0.4")
                .with_body_and_content_length(writer.into_inner())
                .build(),
            Err(err) => {
                HttpResponseBuilder::server_error(format!("failed to encode metrics: {}", err))
                    .build()
            }
        }
    } else if req.path() == "/dashboard" {
        let dashboard = build_dashboard();
        HttpResponseBuilder::ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .with_body_and_content_length(dashboard)
            .build()
    } else {
        HttpResponseBuilder::not_found().build()
    }
}

/// Checks the real candid interface against the one declared in the did file
#[test]
fn check_candid_interface_compatibility() {
    fn source_to_str(source: &candid::utils::CandidSource) -> String {
        match source {
            candid::utils::CandidSource::File(f) => {
                std::fs::read_to_string(f).unwrap_or_else(|_| "".to_string())
            }
            candid::utils::CandidSource::Text(t) => t.to_string(),
        }
    }

    fn check_service_compatible(
        new_name: &str,
        new: candid::utils::CandidSource,
        old_name: &str,
        old: candid::utils::CandidSource,
    ) {
        let new_str = source_to_str(&new);
        let old_str = source_to_str(&old);
        match candid::utils::service_compatible(new, old) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "{} is not compatible with {}!\n\n\
            {}:\n\
            {}\n\n\
            {}:\n\
            {}\n",
                    new_name, old_name, new_name, new_str, old_name, old_str
                );
                panic!("{:?}", e);
            }
        }
    }

    candid::export_service!();

    let new_interface = __export_service();

    // check the public interface against the actual one
    let old_interface =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("core.did");

    check_service_compatible(
        "actual ledger candid interface",
        candid::utils::CandidSource::Text(&new_interface),
        "declared candid interface in core.did file",
        candid::utils::CandidSource::File(old_interface.as_path()),
    );
}
