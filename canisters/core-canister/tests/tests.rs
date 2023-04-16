fn core_wasm() -> Vec<u8> {
    // dfx build core
    // std::fs::read("/home/leo/Code/Elliptic/.dfx/local/canisters/core/core.wasm").unwrap()
    // cargo build --target wasm32-unknown-unknown --release -p core-canister --locked --features=self_check
    // /Users/leo/Code/ellipticdao
    std::fs::read(
        "/Users/leo/Code/ellipticdao/target/wasm32-unknown-unknown/release/core-canister.wasm",
    )
    .unwrap()
}

fn xrc_wasm() -> Vec<u8> {
    // /home/leo/Code/
    std::fs::read("/Users/leo/Code/ic/bazel-bin/rs/rosetta-api/tvl/xrc_mock/xrc_mock_canister.wasm")
        .unwrap()
}

fn icrc1_ledger_wasm() -> Vec<u8> {
    std::fs::read("/Users/leo/Code/ic/bazel-bin/rs/rosetta-api/icrc1/ledger/ledger_canister.wasm")
        .unwrap()
}

#[test]
fn test_core() {
    core_sm_tests::test_core(core_wasm(), xrc_wasm(), icrc1_ledger_wasm())
}

#[test]
fn test_swap() {
    core_sm_tests::test_swap::test_swap(core_wasm(), xrc_wasm(), icrc1_ledger_wasm())
}
