use crate::logs::P0;
use crate::state::eventlog::replay;
use crate::state::eventlog::Event;
use crate::state::replace_state;
use crate::storage::count_events;
use crate::storage::events;
use crate::storage::record_event;
use candid::{CandidType, Deserialize};
use ic_canister_log::log;
use serde::Serialize;

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpgradeArgs {}

pub fn post_upgrade(upgrade_args: Option<UpgradeArgs>) {
    if let Some(upgrade_args) = upgrade_args {
        record_event(&Event::Upgrade(upgrade_args));
    };

    let start = ic_cdk::api::instruction_counter();

    log!(P0, "[upgrade]: replaying {} events", count_events());

    let state = replay(events()).unwrap_or_else(|e| {
        ic_cdk::trap(&format!(
            "[upgrade]: failed to replay the event log: {:?}",
            e
        ))
    });

    replace_state(state);

    let end = ic_cdk::api::instruction_counter();
    log!(
        P0,
        "[upgrade]: replaying events consumed {} instructions",
        end - start
    );
}
