import React, { useEffect, useState } from 'react';
import { use_local_state } from '../../utils/state';
import { open_leverage_position, icrc1_transfer, close_leverage_position } from '../../utils/calls';
import { use_provide_auth, LEDGER_ICP_PRINCIPAL } from '../../utils/auth';


function compute_pnl(entry_price: number, current_price: number, amount: number) {
    const difference = (current_price - entry_price) * (amount / 100_000_000);
    let flat = '';
    if (difference >= 0) {
        flat = `+${difference.toFixed(3)}`
    } else {
        flat = `${difference.toFixed(3)}`
    }
    const ratio = (current_price - entry_price) / entry_price;
    return {
        flat: flat,
        ratio: ratio.toFixed(2),
    }
}

export default function Leverage() {
    const state = use_local_state();
    const auth = use_provide_auth();

    function call_close_leverage_position(deposit_block_index: bigint) {
        const actions: any = [close_leverage_position(deposit_block_index)];
        auth.make_batch_transaction(actions);
    }
    if (state.leverage_positions.length !== 0)
        return (
            <div>
                <NewLeveragePosition></NewLeveragePosition>
                <div style={{ display: 'flex', flexDirection: 'column', marginTop: '1em', textAlign: 'center' }}>
                    <table>
                        <thead>
                            <tr>
                                <th>Opened At</th>
                                <th>Entry Price</th>
                                <th>Collateral Amount</th>
                                <th>Take Profit</th>
                                <th>Leverage</th>
                                <th>Covered Amount</th>
                                <th>P&L</th>
                                <th style={{ display: 'flex', flexDirection: 'column' }}>
                                    <span style={{ fontSize: '12px' }}>Close Position</span>
                                    <span style={{ fontSize: '12px' }}>(At least 1 hour old)</span>
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {state.leverage_positions.map((position, index) => (
                                <tr key={index}>
                                    <td>{new Date(Number(position.timestamp.toString().slice(0, -6))).toLocaleString()}</td>
                                    <td>{(Number(position.icp_entry_price.rate) / Math.pow(10, 8)).toFixed(3)}$</td>
                                    <td>{`${Number(position.amount) / 100_000_000} ICP`}</td>
                                    <td>{`${(Number(position.take_profit) / 100_000_000).toFixed(3)} $`}</td>
                                    <td>{`${((Number(position.amount) + Number(position.covered_amount)) / (Number(position.amount))).toFixed(2)}x`}</td>
                                    <td>{`${Number(position.covered_amount) / 100_000_000} ICP`}</td>
                                    <td>{compute_pnl(Number(position.icp_entry_price.rate) / Math.pow(10, 8), state.icp_price, Number(position.amount)).flat}</td>
                                    <td><button onClick={() => call_close_leverage_position(position.deposit_block_index)}>Close Position</button></td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        );
    else {
        return (<div>
            <NewLeveragePosition></NewLeveragePosition>
        </div>)
    }
}

export function NewLeveragePosition() {
    const state = use_local_state();
    const auth = use_provide_auth();

    const [collateral_amount, set_collateral_amount] = useState<number>(1);
    const [collateral_amount_guard, set_collateral_amount_guard] = useState<number>(1);

    const [take_profit, set_take_profit] = useState<number>(5.3);
    const [take_profit_guard, set_take_profit_guard] = useState<number>(5.3);

    const [covered_amount, set_covered_amount] = useState<number>(1);

    const [leverage, set_leverage] = useState<number>(1.0);
    const [is_loading, set_is_loading] = useState(false);

    function compute_leverage() {
        if (collateral_amount > 0 && covered_amount < state.coverable_amount && collateral_amount <= covered_amount) {
            set_leverage((collateral_amount + covered_amount) / collateral_amount);
        }
    }

    const handleSliderChange = (event) => {
        set_covered_amount(parseFloat(event.target.value));
    };

    function changeColateralAmount(amount) {
        set_collateral_amount_guard(amount);
        if (amount > 0.5 && amount < state.coverable_amount) {
            set_collateral_amount(parseFloat(amount));
        }
    }

    function changeTakeProfitAmount(amount) {
        set_take_profit_guard(amount);
        try {
            const new_amount = parseFloat(amount);
            set_take_profit(new_amount);
        } catch (e) {
            console.log(e);
        }
    }

    async function open_new_position() {
        if (state.deposit_account !== undefined && !is_loading) {
            if (collateral_amount < covered_amount && collateral_amount > 0 && covered_amount > 0 && take_profit > state.icp_price) {
                set_is_loading(true);
                const action_list: any = [];
                const take_profit_e8s = BigInt(Math.floor(take_profit * Math.pow(10, 8)));
                const collateral_amount_e8s = BigInt(parseInt(collateral_amount.toFixed(0)) * Math.pow(10, 8));
                const covered_amount_e8s = BigInt(parseInt(covered_amount.toFixed(0)) * Math.pow(10, 8));
                action_list.push(icrc1_transfer(state.deposit_account, collateral_amount_e8s, LEDGER_ICP_PRINCIPAL));
                console.log(take_profit_e8s, covered_amount_e8s, collateral_amount_e8s);
                action_list.push(open_leverage_position(take_profit_e8s, covered_amount_e8s, collateral_amount_e8s));
                try {
                    await auth.make_batch_transaction(action_list);
                } catch (e) {
                    console.log(e);
                    set_is_loading(false);
                }
                set_is_loading(false);
            }
        }
    }

    useEffect(() => {
        set_take_profit_guard(state.icp_price + 1);
        set_take_profit(state.icp_price + 1);
    }, []);

    useEffect(() => {
        compute_leverage();
    }, [collateral_amount, covered_amount])

    return (
        <div className='leverage'>
            <div style={{ backgroundColor: '#EEF1EF', padding: '1em', margin: 0 }}>
                <b>Open a new leverage position</b>
            </div>
            <div style={{ padding: '1em' }}>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                    <p>Collateral Amount</p>
                    <div style={{ position: 'relative' }}>
                        <input
                            value={collateral_amount_guard}
                            onChange={(e) => changeColateralAmount(e.target.value)}
                            style={{
                                paddingRight: '64px', /* width of image + some padding */
                                boxSizing: 'border-box',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                paddingLeft: '8px',
                                width: '69%',
                            }}
                        />
                        <span style={{ position: 'absolute', top: 0, right: '82px', display: 'flex', alignItems: 'center', height: '100%' }}>
                            ICP
                        </span>
                        <img src='/tokens/icp_logo.png' width='48px'
                            style={{ position: 'absolute', right: '0px', top: 0 }}>
                        </img>
                    </div>
                </ div>
                <div style={{ display: 'flex', marginTop: '1em', alignItems: 'center' }}>
                    <p>Take Profit</p>
                    <div style={{ position: 'relative' }}>
                        <input
                            value={take_profit_guard}
                            onChange={(e) => changeTakeProfitAmount(e.target.value)}
                            style={{
                                paddingRight: '64px', /* width of image + some padding */
                                boxSizing: 'border-box',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                paddingLeft: '8px',
                                width: '69%',
                            }}
                        />
                        <span style={{ position: 'absolute', top: 0, right: '82px', display: 'flex', alignItems: 'center', height: '100%' }}>
                            $
                        </span>
                    </div>
                </div>
                <div style={{ display: 'flex', marginTop: '1em' }}>
                    <p>Leverage : {leverage.toFixed(3)}x</p>
                    <input
                        type="range"
                        min={parseFloat(collateral_amount.toFixed(2))}
                        max={state.coverable_amount}
                        step="0.01"
                        value={covered_amount}
                        onChange={handleSliderChange}
                        style={{ height: '1px', placeSelf: 'center' }}
                    />
                </div>
                <div style={{ marginTop: '1em' }}>
                    <p>Covered Ratio: {(state.covered_ratio * 100).toFixed(2)}%</p>
                </div>
                <div style={{ marginTop: '1em' }}>
                    <b>Coverable Amount: {(state.coverable_amount).toFixed(2)} ICP</b>
                    <p>Positions will be open for at least 10 minutes.</p>
                </div>
                <button onClick={open_new_position} style={{ background: '#0D7CFF', color: 'white', border: 'none', marginLeft: 'auto', display: 'flex', position: 'relative', height: '55px', minWidth: '250px', placeContent: 'center', alignItems: 'center' }}>
                    {is_loading ?
                        <img src="/mobius_strip.gif" style={{
                            height: '55px'
                        }} />
                        :
                        <span>Open Leverage Position</span>
                    }
                </button>
            </div>
        </div >
    );
}