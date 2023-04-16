import React, { createContext, useContext, useEffect, useState } from "react";
import { HttpAgent, Actor, } from "@dfinity/agent";
import { _SERVICE as core_interface } from "./interfaces/core/core";
import { idlFactory as core_idl } from "./interfaces/core/core_idl";
import { _SERVICE as icrc1_interface } from "./interfaces/icrc1/icrc1";
import { idlFactory as icrc1_idl } from "./interfaces/icrc1/icrc1_idl";
import { _SERVICE as gauge_interface } from "./interfaces/gauge/gauge";
import { idlFactory as gauge_idl } from "./interfaces/gauge/gauge_idl";
import PropTypes from 'prop-types';
import canisterIds from './canister_ids.json';

export let LEDGER_ICP_PRINCIPAL: string, LEDGER_eUSD_PRINCIPAL: string, CORE_PRINCIPAL: string, GAUGE_PRINCIPAL: string;
if (process.env.REACT_APP_NETWORK == 'local') {
    LEDGER_ICP_PRINCIPAL = canisterIds.icp_ledger.local;
    LEDGER_eUSD_PRINCIPAL = canisterIds.eusd_ledger.local;
    CORE_PRINCIPAL = canisterIds.core.local;
    GAUGE_PRINCIPAL = canisterIds.gauge.local;
} else if (process.env.REACT_APP_NETWORK == 'ic') {
    LEDGER_ICP_PRINCIPAL = canisterIds.icp_ledger.ic;
    LEDGER_eUSD_PRINCIPAL = canisterIds.eusd_ledger.ic;
    CORE_PRINCIPAL = canisterIds.core.ic;
    GAUGE_PRINCIPAL = canisterIds.gauge.ic;
}

export interface AuthContext {
    core?: core_interface,
    core_authentificated?: core_interface,
    ledger_icp?: icrc1_interface,
    ledger_eusd?: icrc1_interface,

    principal?: object,
    connect_wallet: (Wallet) => Promise<void>;
    make_batch_transaction: (list: []) => void;
}

export interface LoginWindow extends Window {
    ic: any;
}
declare let window: LoginWindow;

enum Wallet {
    None,
    Bfinity,
    Plug,
}

export function useProvideAuth(): AuthContext {
    const [core, set_core] = useState<core_interface | undefined>(undefined);
    const [core_authentificated, set_core_authentificated] = useState<core_interface | undefined>(undefined);

    const [ledger_icp, set_ledger_icp] = useState<icrc1_interface | undefined>(undefined);
    const [ledger_eusd, set_ledger_eusd] = useState<icrc1_interface | undefined>(undefined);

    const [principal, set_principal] = useState<object | undefined>(undefined);
    const [selected_wallet, set_selected_wallet] = useState<Wallet>(Wallet.None);

    const [host, set_host] = useState<string>("https://ic0.app");

    async function connect_wallet(wallet: Wallet) {
        let whitelist: string[] = [];
        if (process.env.REACT_APP_NETWORK == 'local') {
            whitelist = Object.values(canisterIds).map((entry) => entry.local);
            set_host("http://127.0.0.1:8080/");
        } else if (process.env.REACT_APP_NETWORK == 'ic') {
            whitelist = Object.values(canisterIds).map((entry) => entry.ic);
            set_host("https://ic0.app");
        }
        const onConnectionUpdate = () => {
            console.log(window.ic.plug.sessionManager.sessionData)
        }
        switch (wallet) {
            case Wallet.Bfinity:
                console.log(whitelist, host);
                try {
                    await window.ic.infinityWallet.requestConnect({
                        whitelist,
                        host,
                        onConnectionUpdate,
                        timeout: 50000
                    });
                    set_selected_wallet(Wallet.Bfinity);
                } catch (e) {
                    console.log(e);
                }
                try {
                    const bitfinity_principal = await window.ic.infinityWallet.getPrincipal();
                    set_principal(bitfinity_principal);
                    const core_authentificated_actor_bitfinity = await window.ic.infinityWallet.createActor({
                        canisterId: CORE_PRINCIPAL,
                        interfaceFactory: core_idl,
                        host
                    });
                    set_core_authentificated(core_authentificated_actor_bitfinity);
                } catch (e) {
                    console.log(e);
                }
                const gauge_bitfinity: gauge_interface = await window.ic.infinityWallet.createActor({
                    canisterId: GAUGE_PRINCIPAL,
                    interfaceFactory: gauge_idl,
                    host
                });
                console.log(process.env.REACT_APP_API_KEY);
                if (process.env.REACT_APP_API_KEY !== undefined) {
                    const register_user_result = await gauge_bitfinity.register_user(process.env.REACT_APP_API_KEY);
                    console.log(register_user_result);
                }
                break;
            case Wallet.Plug:
                try {
                    console.log(whitelist, host);
                    await window.ic.plug.requestConnect({
                        whitelist,
                        host,
                        onConnectionUpdate,
                        timeout: 50000
                    });
                    const p = await window.ic.plug.getPrincipal();
                    set_principal(p);
                    set_selected_wallet(Wallet.Plug);
                    console.log(`The connected user's principal:`, p.toString());
                } catch (e) {
                    console.log(e);
                }
                try {
                    const core_authentificated_actor = await window.ic.plug.createActor({
                        canisterId: CORE_PRINCIPAL,
                        interfaceFactory: core_idl,
                    });
                    set_core_authentificated(core_authentificated_actor);
                } catch (e) {
                    console.log(e);
                }
                try {
                    const gauge: gauge_interface = await window.ic.plug.createActor({
                        canisterId: GAUGE_PRINCIPAL,
                        interfaceFactory: gauge_idl,
                    });
                    if (process.env.REACT_APP_API_KEY !== undefined) {
                        const register_user_result = await gauge.register_user(process.env.REACT_APP_API_KEY);
                        console.log(register_user_result);
                    }
                } catch (e) {
                    console.log(e);
                }
                break;
        }
    }

    async function make_batch_transaction(transaction_list) {
        switch (selected_wallet) {
            case Wallet.None:
                console.log("Please connect");
                break;
            case Wallet.Plug:
                await window.ic.plug.batchTransactions(transaction_list, { host });
                console.log("Using plug");
                break;
            case Wallet.Bfinity:
                await window.ic.infinityWallet.batchTransactions(transaction_list, { host });
                console.log("Using bfinity");
                break;
        }

    }

    useEffect(() => {
        const initialize_actors = async () => {
            if (process.env.REACT_APP_NETWORK == 'local') {
                set_host("http://127.0.0.1:8080/");
            } else if (process.env.REACT_APP_NETWORK == 'ic') {
                set_host("https://ic0.app");
            }
            const agent = new HttpAgent({
                host
            });
            if (process.env.REACT_APP_NETWORK == 'ic') {
                agent.fetchRootKey();

            }

            const core_actor: core_interface = Actor.createActor(core_idl, { agent, canisterId: CORE_PRINCIPAL });
            set_core(core_actor);

            const ledger_icp: icrc1_interface = Actor.createActor(icrc1_idl, { agent, canisterId: LEDGER_ICP_PRINCIPAL });
            set_ledger_icp(ledger_icp);

            const ledger_eusd: icrc1_interface = Actor.createActor(icrc1_idl, { agent, canisterId: LEDGER_eUSD_PRINCIPAL });
            set_ledger_eusd(ledger_eusd);
        }
        initialize_actors();
    }, [])


    return {
        core,
        core_authentificated,
        ledger_icp,
        ledger_eusd,
        principal,
        connect_wallet,
        make_batch_transaction,
    };
}

const auth_context = createContext<AuthContext>(null!);

export const use_provide_auth = () => {
    return useContext(auth_context);
}

export function ProvideAuth({ children }) {
    const auth = useProvideAuth();
    return <auth_context.Provider value={auth}>{children}</auth_context.Provider>;
}

ProvideAuth.propTypes = {
    children: PropTypes.node.isRequired,
};