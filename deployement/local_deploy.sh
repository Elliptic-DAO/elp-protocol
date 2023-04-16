#!/bin/bash
dfx stop
# rm -r .dfx
dfx start --clean --background

dfx identity use default

# dfx nns install
# dfx nns import

dfx canister create core
dfx canister create icp_ledger
dfx canister create frontend
dfx canister create eusd_ledger
dfx canister create xrc
dfx canister create gauge

dfx build core
dfx build icp_ledger
dfx build eusd_ledger
dfx build frontend
dfx build xrc
dfx build gauge

# cp -f canister_ids.json .dfx/local/
dfx canister install xrc
dfx canister install frontend

export CORE_CANISTER_ID=$(dfx canister id core)
export eUSD_CANISTER_ID=$(dfx canister id eusd_ledger)
export XRC_ID=$(dfx canister id xrc)
export ICP_LEDGER_PRINCIPAL=$(dfx canister id icp_ledger)
export GAUGE_PRINCIPAL=$(dfx canister id gauge)
export CONTROLLER_ID=$(dfx identity get-principal)

dfx canister install gauge --argument '(principal "'${CONTROLLER_ID}'", principal "'${ICP_LEDGER_PRINCIPAL}'")'

dfx canister install core --argument '(variant { Init = record {
  mode = variant { GeneralAvailability };
  eusd_ledger_principal = opt principal "'${eUSD_CANISTER_ID}'";
  xrc_principal = opt principal "'${XRC_ID}'";
  icp_ledger_principal = opt principal "'${ICP_LEDGER_PRINCIPAL}'";
}})' --mode reinstall

dfx canister install icp_ledger --argument '(variant { Init = record {
  token_name = "ICP";
  token_symbol = "ICP";
  fee_collector_account = opt record { owner = principal "'${CORE_CANISTER_ID}'";};
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

dfx canister install eusd_ledger --argument '(variant { Init = record {
  token_name = "eUSD";
  token_symbol = "eUSD";
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

# dfx canister call xrc get_exchange_rate '(record{ base_asset=record{symbol="ICP"; class=variant {Cryptocurrency};}; quote_asset=record{symbol="USD";class=variant{FiatCurrency};}})'
# dfx canister call get_exchange_rate '(record{ base_asset=record{symbol="ICP"; class=Cryptocurrency}; quote_asset=record{symbol="ICP"; class=FiatCurrency};)'
