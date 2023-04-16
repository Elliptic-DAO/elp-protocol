use candid::{CandidType, Encode, Principal};
use core_canister::lifecycle::init::{CoreArgs, InitArgs as CoreInitArgs};
use core_canister::state::Mode;
use ic_base_types::PrincipalId;
use ic_ic00_types::CanisterInstallMode;
use ic_ledger_canister_core::archive::ArchiveOptions;
use ic_state_machine_tests::{CanisterId, StateMachine};
use ic_xrc_types::{Asset as XrcAsset, AssetClass};
use icrc_ledger_types::icrc::generic_metadata_value::MetadataValue as Value;
use icrc_ledger_types::icrc1::account::Account;
use std::time::Duration;
use xrc_mock::{ExchangeRate as ExchangeRateMock, Response, XrcMockInitPayload};

// record { num_blocks_to_archive = 1000; trigger_threshold = 2000; max_message_size_bytes = null; cycles_for_archive_creation = opt 100_000_000_000_000; node_max_memory_size_bytes = opt 3_221_225_472; controller_id = principal "r7inp-6aaaa-aaaaa-aaabq-cai" }
const FEE: u64 = 10_000;
pub const EUSD_FEE: u64 = 1_000_000;
pub const ARCHIVE_TRIGGER_THRESHOLD: u64 = 2000;
pub const NUM_BLOCKS_TO_ARCHIVE: u64 = 1000;
pub const TX_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);
pub const CYCLES_FOR_ARCHIVE_CREATION: u64 = 100_000_000_000_000;
pub const MAX_MESSAGE_SIZE_BYTES: u64 = 3_221_225_472;

#[derive(CandidType, Clone, Debug, PartialEq, Eq)]
pub struct InitArgs {
    pub minting_account: Account,
    pub fee_collector_account: Option<Account>,
    pub initial_balances: Vec<(Account, u64)>,
    pub transfer_fee: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub metadata: Vec<(String, Value)>,
    pub archive_options: ArchiveOptions,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq)]
pub enum ChangeFeeCollector {
    Unset,
    SetTo(Account),
}

#[derive(CandidType, Clone, Debug, Default, PartialEq, Eq)]
pub struct UpgradeArgs {
    pub metadata: Option<Vec<(String, Value)>>,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub transfer_fee: Option<u64>,
    pub change_fee_collector: Option<ChangeFeeCollector>,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq)]
pub enum LedgerArgument {
    Init(InitArgs),
    Upgrade(Option<UpgradeArgs>),
}

pub struct CanisterPrincipals {
    pub core_id: CanisterId,
    pub icp_ledger_id: CanisterId,
    pub eusd_ledger_id: CanisterId,
    pub xrc_id: CanisterId,
}

pub fn setup(
    xrc_wasm: Vec<u8>,
    icrc1_ledger_wasm: Vec<u8>,
    core_canister_wasm: Vec<u8>,
    initial_balances: Vec<(Account, u64)>,
    initial_icp_rate: u64,
) -> (StateMachine, CanisterPrincipals) {
    let env = StateMachine::new();

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
            rate: initial_icp_rate,
        }),
    };

    let xrc_args = Encode!(&xrc_args).unwrap();
    let xrc_id = env.install_canister(xrc_wasm, xrc_args, None).unwrap();

    let eusd_ledger_id = env.create_canister(None);
    let icp_ledger_id = env.create_canister(None);

    install_icp_ledger(
        &env,
        icrc1_ledger_wasm.clone(),
        icp_ledger_id,
        initial_balances,
    );

    let core_id = install_core_canister(
        &env,
        core_canister_wasm,
        Some(eusd_ledger_id.into()),
        Some(xrc_id.into()),
        Some(icp_ledger_id.into()),
    );

    install_eusd_ledger(&env, icrc1_ledger_wasm, eusd_ledger_id, core_id);

    let cp = CanisterPrincipals {
        core_id,
        icp_ledger_id,
        eusd_ledger_id,
        xrc_id,
    };
    (env, cp)
}

fn install_core_canister(
    env: &StateMachine,
    core_canister_wasm: Vec<u8>,
    eusd_ledger_principal: Option<Principal>,
    xrc_principal: Option<Principal>,
    icp_ledger_principal: Option<Principal>,
) -> CanisterId {
    let init_args = CoreInitArgs {
        mode: Mode::NoHttpOutCalls,
        eusd_ledger_principal,
        xrc_principal,
        icp_ledger_principal,
        min_amount_to_stable: None,
        min_amount_from_stable: None,
        min_amount_leverage: None,
        min_amount_liquidity: None,
    };
    let core_args = CoreArgs::Init(init_args);
    let args = Encode!(&core_args).unwrap();
    env.install_canister(core_canister_wasm, args, None)
        .unwrap()
}

fn install_icp_ledger(
    env: &StateMachine,
    icrc1_ledger_wasm: Vec<u8>,
    icp_ledger_id: CanisterId,
    initial_balances: Vec<(Account, u64)>,
) {
    let init_args = InitArgs {
        minting_account: Account {
            owner: PrincipalId::new(0, [0u8; 29]).0,
            subaccount: None,
        },
        fee_collector_account: None,
        initial_balances,
        transfer_fee: FEE,
        token_name: "ICP".into(),
        token_symbol: "ICP".into(),
        metadata: vec![],
        archive_options: ArchiveOptions {
            trigger_threshold: ARCHIVE_TRIGGER_THRESHOLD as usize,
            num_blocks_to_archive: NUM_BLOCKS_TO_ARCHIVE as usize,
            node_max_memory_size_bytes: None,
            max_message_size_bytes: Some(MAX_MESSAGE_SIZE_BYTES),
            controller_id: PrincipalId::new_user_test_id(100),
            cycles_for_archive_creation: Some(CYCLES_FOR_ARCHIVE_CREATION),
            max_transactions_per_response: None,
        },
    };
    let ledger_arg = LedgerArgument::Init(init_args);
    let args = Encode!(&ledger_arg).unwrap();
    env.install_wasm_in_mode(
        icp_ledger_id,
        CanisterInstallMode::Install,
        icrc1_ledger_wasm,
        args,
    )
    .unwrap();
}

fn install_eusd_ledger(
    env: &StateMachine,
    icrc1_ledger_wasm: Vec<u8>,
    eusd_ledger_id: CanisterId,
    core_id: CanisterId,
) {
    let init_args = InitArgs {
        minting_account: Account {
            owner: core_id.into(),
            subaccount: None,
        },
        fee_collector_account: None,
        initial_balances: vec![],
        transfer_fee: EUSD_FEE,
        token_name: "Elliptic USD".into(),
        token_symbol: "eUSD".into(),
        metadata: vec![],
        archive_options: ArchiveOptions {
            trigger_threshold: ARCHIVE_TRIGGER_THRESHOLD as usize,
            num_blocks_to_archive: NUM_BLOCKS_TO_ARCHIVE as usize,
            node_max_memory_size_bytes: None,
            max_message_size_bytes: Some(MAX_MESSAGE_SIZE_BYTES),
            controller_id: PrincipalId::new_user_test_id(100),
            cycles_for_archive_creation: Some(CYCLES_FOR_ARCHIVE_CREATION),
            max_transactions_per_response: None,
        },
    };
    let ledger_arg = LedgerArgument::Init(init_args);
    let args = Encode!(&ledger_arg).unwrap();
    env.install_wasm_in_mode(
        eusd_ledger_id,
        CanisterInstallMode::Install,
        icrc1_ledger_wasm,
        args,
    )
    .unwrap();
}
