import React from "react";
import { useEffect, useState } from "react";
import SocialNetworks from "./social_network";
import { use_local_state } from "../utils/state";
import { use_provide_auth } from "../utils/auth";

function display_principal(principal) {
    let a = principal.split('-')
    return a[0] + '...' + a[a.length - 1]
}

export default function Navbar() {
    let state = use_local_state();
    let auth = use_provide_auth();

    const handleClick = (value) => {
        state.set_active_component(value);
    };

    return (
        <nav>
            <img src="/mobius_strip.png" />
            <h1>Elliptic DAO</h1>
            <button onClick={() => handleClick('swap')}
                style={{ marginLeft: '8em' }}
                className={state.active_component === 'swap' ? 'active' : ''}>
                Swap
            </button>
            <button onClick={() => handleClick('leverage')}
                className={state.active_component === 'leverage' ? 'active' : ''}>
                Leverage
            </button>
            <button onClick={() => handleClick('liquidity')}
                className={state.active_component === 'liquidity' ? 'active' : ''}>
                Liquidity
            </button>
            <div>
                {/* <button onClick={() => handleClick('statistics')}
                    className={state.active_component === 'statistics' ? 'active' : ''}>
                    Analytics
                </button> */}
                <a href="/elliptic_dao_whitepaper.pdf" target="_blank" style={{ alignSelf: 'center', fontSize: '14px', fontWeight: 'bold' }}>White Paper</a>
                {auth.principal === undefined ?
                    <button onClick={() => handleClick('auth')}
                        className={state.active_component === 'auth' ? 'active' : ''}
                        style={{
                            fontWeight: 'bold', color: 'rgb(13, 124, 255)'
                        }}>
                        Connect
                    </button>
                    :
                    <div style={{ alignItems: 'center', display: 'flex', flexDirection: 'column', alignSelf: 'center' }}>
                        <button style={{ height: 'auto' }}>
                            <b>{display_principal(auth.principal.toString())}</b>
                            <p>{state.icp_balance.toFixed(2)} ICP</p>
                            <p>{state.eusd_balance.toFixed(2)} eUSD</p>
                        </button>
                    </div>
                }

            </div>
        </nav >
    );
}