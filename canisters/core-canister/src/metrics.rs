use crate::state;

pub fn encode_metrics(
    metrics: &mut ic_metrics_encoder::MetricsEncoder<Vec<u8>>,
) -> std::io::Result<()> {
    const WASM_PAGE_SIZE_IN_BYTES: f64 = 65536.0;

    metrics.encode_gauge(
        "core_stable_memory_bytes",
        ic_cdk::api::stable::stable_size() as f64 * WASM_PAGE_SIZE_IN_BYTES,
        "Size of the stable memory allocated by this canister.",
    )?;
    metrics.encode_gauge(
        "core_cycle_balance",
        ic_cdk::api::canister_balance128() as f64,
        "Cycle balance on this canister.",
    )?;

    metrics.encode_gauge(
        "core_leverage_total_amount",
        state::read_state(|s| s.get_total_leverage_amount() as f64),
        "The total amount in leverage positions.",
    )?;

    metrics.encode_gauge(
        "core_liquidity_total_amount",
        state::read_state(|s| s.get_total_liquidity_amount() as f64),
        "The total amount in liquidity positions.",
    )?;

    metrics.encode_gauge(
        "core_eusd_total_minted",
        state::read_state(|s| s.total_eusd_minted as f64),
        "The total amount of eusd that has been minted.",
    )?;

    metrics.encode_gauge(
        "core_eusd_total_burned",
        state::read_state(|s| s.total_eusd_burned as f64),
        "The total amount of eusd that has been burned.",
    )?;

    metrics.encode_gauge(
        "core_collateral_ratio",
        state::read_state(|s| s.get_collateral_ratio() as f64),
        "The collateral ratio of the Elliptic Protocol.",
    )?;

    metrics.encode_gauge(
        "core_protocol_balance",
        state::read_state(|s| s.protocol_balance as f64),
        "The ICP balance of all the Elliptic Protocol.",
    )?;

    metrics.encode_gauge(
        "core_collateral_amount",
        state::read_state(|s| s.icp_collateral_amount as f64),
        "The collateral amount of the swap.",
    )?;

    metrics.encode_gauge(
        "core_liquidity_amount",
        state::read_state(|s| s.icp_liqudity_amount as f64),
        "The liquidity amount provided to the protocol.",
    )?;

    metrics.encode_gauge(
        "core_leverage_margin_amount",
        state::read_state(|s| s.icp_leverage_margin_amount as f64),
        "The leverage margin amount provided to the protocol.",
    )?;

    metrics.encode_gauge(
        "core_total_claimable_rewards",
        state::read_state(|s| {
            (s.liquidity_rewards.values().sum::<u64>() + s.total_available_fees) as f64
        }),
        "The total reward amount claimable by liquidity providers.",
    )?;

    Ok(())
}
