use super::{eventlog::Event, LeveragePosition};
use crate::state::CoreState;
use crate::state::IcpPrice;
use crate::storage::record_event;
use crate::updates::liquidity::{Liquidity, LiquidityType};
use crate::updates::swap::{Swap, SwapSuccess};

pub fn record_swap(state: &mut CoreState, swap: Swap) {
    record_event(&Event::Swap(swap.clone()));
    state.open_swaps.insert(swap.from_block_index, swap.clone());
    state.distribute_fee(swap.fee);
}

pub fn record_swap_success(state: &mut CoreState, from_block_index: u64, to_block_index: u64) {
    record_event(&Event::SwapSuccess(SwapSuccess {
        from_block_index,
        to_block_index,
    }));
    state.finish_swap(from_block_index);
    state.distribute_fee(0);
}

pub fn record_open_leverage_position(state: &mut CoreState, leverage_position: LeveragePosition) {
    record_event(&Event::OpenLeveragePosition(leverage_position.clone()));
    state.open_leverage_position(leverage_position.clone());
    state.distribute_fee(leverage_position.fee);
}

pub fn record_close_leverage_position(
    state: &mut CoreState,
    output_block_index: u64,
    deposit_block_index: u64,
    fee: u64,
    timestamp: u64,
    icp_price: IcpPrice,
) {
    record_event(&Event::CloseLeveragePosition {
        deposit_block_index,
        output_block_index: Some(output_block_index),
        fee,
        timestamp,
        icp_price: icp_price.clone(),
    });
    if let Some(leverage_position_to_remove) = state.get_leverage_position(deposit_block_index) {
        state.close_leverage_position(leverage_position_to_remove, icp_price, fee);
    } else {
        panic!("inconsistent state, cannot close leverage position");
    }
    state.distribute_fee(fee);
}

pub fn record_liquidate_leverage_position(
    state: &mut CoreState,
    deposit_block_index: u64,
    fee: u64,
    timestamp: u64,
    icp_price: IcpPrice,
) {
    record_event(&Event::CloseLeveragePosition {
        deposit_block_index,
        output_block_index: None,
        fee,
        icp_price,
        timestamp,
    });

    if let Some(leverage_position_to_remove) = state.get_leverage_position(deposit_block_index) {
        if let Some(user_positions) = state
            .leverage_positions
            .get_mut(&leverage_position_to_remove.owner)
        {
            user_positions.remove(&leverage_position_to_remove);
        }
    }
    state.distribute_fee(fee);
}

pub fn record_liquidity(state: &mut CoreState, liquidity: Liquidity) {
    record_event(&Event::Liquidity(liquidity.clone()));

    match liquidity.operation_type {
        LiquidityType::Add => state.add_liquidity(&liquidity),
        LiquidityType::Remove => state.remove_liquidity(&liquidity),
    }
    state.distribute_fee(liquidity.fee);
}
