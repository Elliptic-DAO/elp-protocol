import React, { useState } from 'react';
import { use_local_state } from '../../utils/state';
import { icrc1_transfer, liquidity_operation, claim_liquidity_rewards } from '../../utils/calls';
import { use_provide_auth, LEDGER_ICP_PRINCIPAL } from '../../utils/auth';

enum Operation {
    Add,
    Remove
}

export default function Liquidity() {
    const state = use_local_state();
    const auth = use_provide_auth();

    const [amount, set_amount] = useState<number>(1);
    const [guard_amount, set_guard_amount] = useState(1);
    const [is_loading_add, set_is_loading_add] = useState(false);
    const [is_loading_remove, set_is_loading_remove] = useState(false);

    function on_change_amount(amount) {
        try {
            set_amount(parseFloat(amount));
        } catch (e) {
            console.log(e);
        }
        set_guard_amount(amount);
    }

    async function make_liquidity_operation(operation_type: Operation) {
        if (state.deposit_account !== undefined && !is_loading_add && !is_loading_remove) {
            if (amount > 0) {
                const amount_e8s = BigInt(amount * Math.pow(10, 8));
                const action_list: any = [];
                switch (operation_type) {
                    case Operation.Add:
                        set_is_loading_add(true);
                        action_list.push(icrc1_transfer(state.deposit_account, BigInt(amount_e8s), LEDGER_ICP_PRINCIPAL));
                        action_list.push(liquidity_operation(operation_type, BigInt(amount_e8s)));
                        try {
                            await auth.make_batch_transaction(action_list);
                        } catch (e) {
                            set_is_loading_add(false);
                        }
                        set_is_loading_add(false)
                        break;
                    case Operation.Remove:
                        set_is_loading_remove(true);
                        action_list.push(liquidity_operation(operation_type, BigInt(amount_e8s)));
                        try {
                            await auth.make_batch_transaction(action_list);
                        } catch (e) {
                            set_is_loading_remove(false);
                        }
                        set_is_loading_remove(false);
                        break;
                }
            }
        }
    }

    function call_claim_liquidity_rewards() {
        if (state.deposit_account !== undefined) {
            const action_list: any = [];
            action_list.push(claim_liquidity_rewards());
            auth.make_batch_transaction(action_list);
        }
    }

    function set_maximum_add_liquidity() {
        set_guard_amount(state.icp_balance);
        set_amount(state.icp_balance);
    }

    return (
        <div>
            <div className='liquidity'>
                <div style={{ backgroundColor: '#EEF1EF', padding: '1em', margin: 0 }}>
                    <b>Manage Liquidity</b>
                </div>
                <div style={{ padding: '1em', marginTop: '6px' }}>
                    <div style={{ position: 'relative' }}>
                        <input
                            value={guard_amount}
                            onChange={(event) => on_change_amount(event.target.value)}
                            style={{
                                paddingRight: '64px', /* width of image + some padding */
                                boxSizing: 'border-box',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                paddingLeft: '8px',
                            }}
                        />
                        <span style={{ position: 'absolute', top: 0, right: '160px', display: 'flex', alignItems: 'center', height: '100%' }}>
                            <button style={{ fontSize: 'small', background: 'rgb(13, 124, 255)', color: 'white', borderRadius: '5px' }}
                                onClick={set_maximum_add_liquidity}
                            >
                                MAX
                            </button>
                        </span>
                        <span style={{ position: 'absolute', top: 0, right: '135px', display: 'flex', alignItems: 'center', height: '100%' }}>
                            ICP
                        </span>
                        <img src='/tokens/icp_logo.png' width='48px'
                            style={{ position: 'absolute', right: '36px', top: 0 }}>
                        </img>
                    </div>
                    <p style={{
                        marginLeft: 'auto',
                        display: 'table',
                        marginTop: '1.2em',
                        fontStyle: 'italic'
                    }}>Liquidity Balance: {state.liquidity_provided.toFixed(2)} ICP</p>
                    <div style={{ display: 'flex' }}>
                        <button style={{ color: '#0D7CFF', background: 'white', border: 'solid', position: 'relative', width: 'fit-content', height: '55px', minWidth: '80px', marginLeft: 'auto' }} onClick={() => make_liquidity_operation(Operation.Remove)}>
                            {is_loading_remove ?
                                <>
                                    <img src="/mobius_strip.gif" style={{ position: 'absolute', top: 0, left: '12px', width: '70%', height: '100%' }} />
                                </>
                                :
                                <span style={{ zIndex: 1 }}>Remove</span>
                            }
                        </button>
                        <button style={{ background: '#0D7CFF', color: 'white', border: 'none', position: 'relative', width: 'fit-content', height: '55px', minWidth: '80px' }} onClick={() => make_liquidity_operation(Operation.Add)}>
                            {is_loading_add ?
                                <>
                                    <img src="/mobius_strip.gif" style={{ position: 'absolute', top: 0, left: '12px', width: '70%', height: '100%' }} />
                                </>
                                :
                                <span style={{ zIndex: 1 }}>Add</span>
                            }
                        </button>

                    </div>

                </div>
            </div>
            <div className='liquidity' style={{ marginTop: '1em' }}>
                <div style={{ backgroundColor: '#EEF1EF', padding: '1em', margin: 0 }}>
                    <b>Claim Liquidity rewards</b>
                </div>
                <div style={{ padding: '1em' }}>
                    <p>Liquidity Provided: {state.liquidity_provided.toFixed(2)} ICP</p>
                    <p>Claimable Reward: {state.liquidity_reward.toFixed(2)} ICP</p>
                </div>
                <button style={{ marginLeft: 'auto' }} onClick={call_claim_liquidity_rewards}>Claim Liquidity Reward</button>
            </div>
        </div >

    );
}