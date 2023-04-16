dfx build --network ic core
dfx build --network ic icp_ledger
dfx build --network ic eusd_ledger
dfx build --network ic frontend
dfx build --network ic gauge

dfx canister --network ic install frontend

export CORE_CANISTER_ID=$(dfx canister --network ic id core)
export eUSD_CANISTER_ID=$(dfx canister --network ic id eusd_ledger)
export XRC_ID=$(dfx canister --network ic id xrc)
export ICP_LEDGER_PRINCIPAL=$(dfx canister --network ic id icp_ledger)
export GAUGE_PRINCIPAL=$(dfx canister --network ic id gauge)
export CONTROLLER_ID=$(dfx identity get-principal)

dfx canister --network ic install gauge --argument '(principal "'${CONTROLLER_ID}'", principal "'${ICP_LEDGER_PRINCIPAL}'")'

dfx canister --network ic install core --argument '(variant { Init = record {
  mode = variant { GeneralAvailability };
  eusd_ledger_principal = opt principal "'${eUSD_CANISTER_ID}'";
  xrc_principal = opt principal "'${XRC_ID}'";
  icp_ledger_principal = opt principal "'${ICP_LEDGER_PRINCIPAL}'";
}})' --mode reinstall

dfx canister --network ic install icp_ledger --argument '(variant { Init = record {
  token_name = "ICP";
  token_symbol = "ICP";
  minting_account = record { owner = principal "'${GAUGE_PRINCIPAL}'";};
  initial_balances = vec {};
  metadata = vec {};
  transfer_fee = 10;
  archive_options = record {
    trigger_threshold = 2000:nat64;
    num_blocks_to_archive = 1000:nat64;
    controller_id = principal "'${CONTROLLER_ID}'";
  }
}})'

dfx canister --network ic install eusd_ledger --argument '(variant { Init = record {
  token_name = "eUSD";
  token_symbol = "eUSD";
  fee_collector_account = opt record { owner = principal "'${CORE_CANISTER_ID}'";};
  minting_account = record { owner = principal "'${CORE_CANISTER_ID}'";};
  initial_balances = vec {};
  metadata = vec {};
  transfer_fee = 10_000_000;
  archive_options = record {
    trigger_threshold = 2000:nat64;
    num_blocks_to_archive = 1000:nat64;
    controller_id = principal "'${CONTROLLER_ID}'";
  }
}})'