import { use_provide_auth } from "../utils/auth";
import { use_local_state } from "../utils/state";
import React from "react";

enum Wallet {
    None,
    Bfinity,
    Plug,
}

export default function Login() {
    const auth = use_provide_auth();
    const state = use_local_state();

    async function connect_with_plug() {
        state.set_is_loading(true);
        auth.connect_wallet(Wallet.Plug).then(() => {
            state.set_is_loading(false);
        })
    }

    async function connect_with_bfinity() {
        state.set_is_loading(true);
        auth.connect_wallet(Wallet.Bfinity).then(() => {
            state.set_is_loading(false);
        })
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', border: 'solid', width: '333px', height: '333px', placeContent: 'center' }}>
            {state.is_loading ?
                <img style={{ width: '260px' }} src="/mobius_strip.gif"></img>
                :
                <>
                    <div style={{ display: 'flex', alignItems: 'center' }}>
                        <img src="/bitfinity.png" width='30px'></img>
                        <button onClick={connect_with_bfinity} style={{ fontWeight: 'bold' }}>
                            Bfinity Wallet
                        </button>

                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', marginTop: '1em' }}>
                        <img src="/plug.svg" width='30px'></img>
                        <button onClick={connect_with_plug} style={{ fontWeight: 'bold' }}>
                            Plug Wallet
                        </button>
                    </div>
                </>
            }
        </div >
    );
}