import React, { useEffect, useState } from 'react';
import { use_local_state } from '../../utils/state';
import { use_provide_auth, LEDGER_ICP_PRINCIPAL, LEDGER_eUSD_PRINCIPAL } from '../../utils/auth';
import { icrc1_transfer, convert_icp_to_eusd, convert_eusd_to_icp } from '../../utils/calls';

enum Asset {
    ICP,
    eUSD
}

function assetToString(asset: Asset): string {
    switch (asset) {
        case Asset.ICP:
            return "ICP ";
        case Asset.eUSD:
            return "eUSD";
        default:
            throw new Error(`Unknown asset: ${asset}`);
    }
}

function floatToBigInt(num: number): bigint {
    return BigInt(Math.round(num * 1e8));
}

export default function Swap() {
    const state = use_local_state();
    const auth = use_provide_auth();

    const [is_loading, set_is_loading] = useState(false);
    const [from, set_from_asset] = useState(Asset.ICP);
    const [to, set_to_asset] = useState(Asset.eUSD);
    const [from_amount, set_from_amount] = useState<number>(1.000);
    const [from_amount_guard, set_from_amount_guard] = useState<number>(1.000);
    const [to_amount, set_to_amount] = useState<number>(0);
    const [isHovered, setIsHovered] = useState(false);

    const handleMouseEnter = () => {
        setIsHovered(true);
    };

    const handleMouseLeave = () => {
        setIsHovered(false);
    };

    async function make_transactions() {
        if (state.deposit_account !== undefined && !is_loading) {
            set_is_loading(true);
            const amount_to_swap = BigInt(floatToBigInt(from_amount));
            const icp_transfer_fee = BigInt(10000);
            const list_of_transaction: any = [];
            try {
                switch (from) {
                    case Asset.ICP:
                        list_of_transaction.push(icrc1_transfer(state.deposit_account, amount_to_swap, LEDGER_ICP_PRINCIPAL));
                        list_of_transaction.push(convert_icp_to_eusd(amount_to_swap - icp_transfer_fee));
                        await auth.make_batch_transaction(list_of_transaction);
                        set_is_loading(false);
                        break;
                    case Asset.eUSD:
                        list_of_transaction.push(icrc1_transfer(state.deposit_account, amount_to_swap, LEDGER_eUSD_PRINCIPAL));
                        list_of_transaction.push(convert_eusd_to_icp(amount_to_swap - icp_transfer_fee));
                        await auth.make_batch_transaction(list_of_transaction);
                        set_is_loading(false);
                        break;
                }
            } catch (e) {
                console.log(e);
                set_is_loading(false);
            }
        } else {
            set_is_loading(false);
        }
    }

    function invert_conversion_order() {
        switch (from) {
            case Asset.ICP:
                set_from_asset(Asset.eUSD);
                set_to_asset(Asset.ICP);
                break;
            case Asset.eUSD:
                set_from_asset(Asset.ICP);
                set_to_asset(Asset.eUSD);
                break;
        }
    }

    function get_token_url(asset: Asset) {
        switch (asset) {
            case Asset.ICP:
                return "/tokens/icp_logo.png"
            case Asset.eUSD:
                return "tokens/eusd_logo.png"
        }
    }

    function compute_arrival_price(target) {
        switch (from) {
            case Asset.ICP:
                set_to_amount(target * state.icp_price);
                break;
            case Asset.eUSD:
                set_to_amount(target * 1 / state.icp_price);
                break;
        }
    }

    function set_maximum_from_amount() {
        switch (from) {
            case Asset.ICP:
                set_from_amount(state.icp_balance);
                set_from_amount_guard(state.icp_balance);
                break;
            case Asset.eUSD:
                set_from_amount(state.eusd_balance);
                set_from_amount_guard(state.eusd_balance);
                break;
        }
    }

    function change_from_amount(amount) {
        set_from_amount_guard(amount);
        try {
            set_from_amount(parseFloat(amount));
        } catch (e) {
            console.log(e);
        }
    }
    const rotationDegree = isHovered ? 180 : 0;

    useEffect(() => {
        set_from_amount(from_amount);
        compute_arrival_price(from_amount);
    }, [from_amount, from, state.icp_price])

    return (
        <div className='convert' style={{ display: 'flex', placeContent: 'center', flexDirection: 'column', border: 'solid' }}>
            <div style={{ backgroundColor: '#EEF1EF', padding: '1em' }}>
                <b>Swap {assetToString(from)} to {assetToString(to)}</b>
            </div>
            <div style={{ alignItems: 'left', display: 'flex', flexDirection: 'column', padding: '1em' }}>
                <div style={{ display: 'flex', flex: 1, marginTop: '1em' }}>
                    <div style={{ position: 'relative' }}>
                        <input
                            value={from_amount_guard}
                            onChange={(e) => change_from_amount(e.target.value)}
                            style={{
                                paddingRight: '64px', /* width of image + some padding */
                                boxSizing: 'border-box',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                paddingLeft: '8px',
                            }}
                        />
                        <span style={{ position: 'absolute', top: 0, right: '45px', display: 'flex', alignItems: 'center', height: '100%' }}>
                            <button style={{ fontSize: 'small', background: 'rgb(13, 124, 255)', color: 'white', borderRadius: '5px' }}
                                onClick={set_maximum_from_amount}>
                                MAX
                            </button>
                        </span>
                        <span style={{ position: 'absolute', top: 0, right: '15px', display: 'flex', alignItems: 'center', height: '100%' }}>
                            {assetToString(from)}
                        </span>
                        <img src={get_token_url(from)} width='48px'
                            style={{ position: 'absolute', right: '-65px', top: '0px', border: 'solid', borderRadius: '100%' }}>
                        </img>
                    </div>
                </div>

                <button onClick={invert_conversion_order} style={{ border: 'none', width: '40px' }}>
                    <div
                        style={{
                            transition: 'transform 0.5s ease',
                            transform: `rotate(${rotationDegree}deg)`,
                            transformOrigin: 'center',
                            display: 'inline-block'
                        }}
                        onMouseEnter={handleMouseEnter}
                        onMouseLeave={handleMouseLeave}
                    >
                        <img src="/icon/exchange.png" style={{ width: '100%', height: '100%' }} />
                    </div>
                </button>

                <div style={{ position: 'relative', marginTop: '1em' }}>
                    <input
                        value={to_amount}
                        disabled={true}
                        style={{
                            paddingRight: '64px', /* width of image + some padding */
                            boxSizing: 'border-box',
                            border: '1px solid #ccc',
                            borderRadius: '4px',
                            paddingLeft: '8px',
                        }}
                    />
                    <span style={{ position: 'absolute', top: 0, right: '138px', display: 'flex', alignItems: 'center', height: '100%' }}>
                        {assetToString(to)}
                    </span>
                    <img src={get_token_url(to)} width='48px'
                        style={{ position: 'absolute', right: '36px', top: 0, border: 'solid', borderRadius: '100%' }}>
                    </img>
                </div>

                <button onClick={make_transactions} style={{ background: '#0D7CFF', color: 'white', border: 'none', marginTop: '2em', height: '55px', minWidth: '80px', position: 'relative' }}>
                    {is_loading ?
                        <>
                            <img src="/mobius_strip.gif" style={{ position: 'absolute', top: 0, left: '12px', width: '70%', height: '100%' }} />

                        </>
                        :
                        <span>Swap</span>
                    }

                </button>


            </div>
        </div >
    );
}