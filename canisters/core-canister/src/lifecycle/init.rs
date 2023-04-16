use crate::lifecycle::upgrade::UpgradeArgs;
use crate::state::Mode;
use crate::state::{replace_state, CoreState};
use candid::Principal;
use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(candid::CandidType, serde::Deserialize)]
pub enum CoreArgs {
    Init(InitArgs),
    Upgrade(Option<UpgradeArgs>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InitArgs {
    pub mode: Mode,
    pub eusd_ledger_principal: Option<Principal>,
    pub xrc_principal: Option<Principal>,
    pub icp_ledger_principal: Option<Principal>,

    pub min_amount_to_stable: Option<u64>,
    pub min_amount_from_stable: Option<u64>,
    pub min_amount_leverage: Option<u64>,
    pub min_amount_liquidity: Option<u64>,
}

pub fn init(args: InitArgs) {
    replace_state(CoreState::from(args));
}
