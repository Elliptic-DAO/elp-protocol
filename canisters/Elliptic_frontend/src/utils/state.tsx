import React, { createContext, useContext, useEffect, useState } from "react";
import { use_provide_auth } from "./auth";
import { Account } from "./interfaces/icrc1/icrc1";
import { Principal } from '@dfinity/principal';
import { LeveragePosition, ProtocolStatus, UserData } from './interfaces/core/core';

const E8S = 100_000_000;

export interface StateContext {
    active_component: string;
    set_active_component: (component_name: string) => void;
    refresh_balances: () => void;

    deposit_account?: Account;

    leverage_positions: Array<LeveragePosition>;
    collateral_ratio: number;
    tvl: number;
    icp_price: number;
    coverable_amount: number;
    covered_ratio: number;

    liquidity_provided: number;
    liquidity_reward: number;

    icp_balance: number;
    eusd_balance: number;

    is_loading: boolean;
    set_is_loading: (b: boolean) => void;
}

export function useProvideState(): StateContext {
    const auth = use_provide_auth();

    const [is_loading, set_is_loading_state] = useState(false);
    const [liquidity_reward, set_liquidity_reward] = useState(0);
    const [liquidity_provided, set_liquidity_provided] = useState(0);
    const [leverage_positions, set_leverage_positions] = useState<Array<LeveragePosition>>([]);
    const [active_component, set_active_component] = useState('swap');
    const [collateral_ratio, set_collateral_ratio] = useState(0);
    const [tvl, set_tvl] = useState(0);
    const [covered_ratio, set_covered_ratio] = useState(0);
    const [icp_price, set_icp_price] = useState(0);
    const [coverable_amount, set_coverable_amount] = useState(0);
    const [icp_balance, set_icp_balance] = useState(0);
    const [eusd_balance, set_eusd_balance] = useState(0);
    const [deposit_account, set_deposit_account] = useState<undefined | Account>(undefined);

    function set_is_loading(b: boolean) {
        set_is_loading_state(b);
    }

    async function fetch_protocol_status() {
        if (auth.core) {
            console.log("Refreshing Protocol Status");
            const protocol_status: ProtocolStatus = await auth.core.get_protocol_status();
            try {
                set_collateral_ratio(Number(protocol_status.collateral_ratio) / E8S);
                set_icp_price(Number(protocol_status.icp_price) / E8S);
                set_covered_ratio(Number(protocol_status.coverered_ratio) / E8S);
                set_tvl(Number(protocol_status.tvl) / E8S);
                set_coverable_amount(Number(protocol_status.coverable_amount) / E8S);
            } catch (e) {
                console.log(e);
            }
        }
    }

    const fetchData = async (fetchFunc) => {
        let retries = 0;
        const retryLimit = 3;
        const retryDelay = 1000; // 1 second
        try {
            await fetchFunc();
        } catch (error) {
            console.log(error);
            if (retries < retryLimit) {
                retries++;
                await new Promise((resolve) => setTimeout(resolve, retryDelay));
                await fetchData(fetchFunc);
            } else {
                console.error(`Failed to fetch data after ${retries} retries: ${error}`);
            }
        }
    };

    useEffect(() => {
        if (auth.core) {
            const fetchAllData = async () => {
                await Promise.all([
                    fetchData(fetch_protocol_status),
                ]);
            };

            const intervalId = setInterval(fetchAllData, 10000); // fetch every 20 seconds

            fetchAllData(); // fetch data initially

            return () => {
                clearInterval(intervalId); // cleanup interval when component unmounts
            };
        }
    }, [auth.core]);

    useEffect(() => {
        if (auth.principal !== undefined) {
            set_active_component('swap');
        }
    }, [auth.principal]);

    async function fetch_icp_balance() {
        if (auth.principal !== undefined && auth.ledger_icp !== undefined) {
            const principal = Principal.fromText(auth.principal.toString());
            const user_account: Account = {
                'owner': principal,
                'subaccount': [],
            };
            const result = await auth.ledger_icp.icrc1_balance_of(user_account);
            const balance = parseInt(result.toString()) / Math.pow(10, 8);
            set_icp_balance(balance)
            console.log(balance);
        }
    }

    async function fetch_eusd_balance() {
        if (auth.principal !== undefined && auth.ledger_eusd !== undefined) {
            const principal = Principal.fromText(auth.principal.toString());
            const user_account: Account = {
                'owner': principal,
                'subaccount': [],
            };
            const result = await auth.ledger_eusd.icrc1_balance_of(user_account);
            const balance = parseInt(result.toString()) / Math.pow(10, 8);
            set_eusd_balance(balance)
            console.log(balance);
        }
    }

    async function fetch_user_data() {
        if (auth.principal !== undefined && auth.core !== undefined) {
            console.log("Fetching user data.")
            const user_data: UserData = await auth.core.get_user_data(Principal.fromText(auth.principal.toString()));
            try {
                set_liquidity_provided(Number(user_data.liquidity_provided) / E8S);
                set_liquidity_reward(Number(user_data.claimable_liquidity_rewards) / E8S);
                if (user_data.leverage_positions.length != 0) {
                    set_leverage_positions(user_data.leverage_positions[0]);
                }
            } catch (e) {
                console.log(e);
            }
        }
    }

    const refresh_balances = async () => {
        await Promise.all([
            fetchData(fetch_icp_balance),
            fetchData(fetch_eusd_balance),
            fetchData(fetch_user_data),
        ]);
    }

    useEffect(() => {
        console.log("Fetching balances");
        const intervalId = setInterval(refresh_balances, 5000); // fetch every 5 seconds

        refresh_balances(); // fetch data initially

        return () => {
            clearInterval(intervalId); // cleanup interval when component unmounts
        };
    }, [auth.principal])


    useEffect(() => {
        fetch_icp_balance();
    }, [auth.ledger_icp, auth.principal])

    useEffect(() => {
        fetch_eusd_balance();
    }, [auth.ledger_eusd, auth.principal])


    useEffect(() => {
        async function fetch_deposit_account() {
            if (auth.core_authentificated) {
                const res = await auth.core_authentificated.get_deposit_account();
                const core_principal = res.owner;
                const subaccount = res.subaccount[0];
                set_deposit_account(res);
                console.log(core_principal.toString(), subaccount);
            }
        }
        fetch_deposit_account();
    }, [auth.core_authentificated]);

    return {
        liquidity_reward,
        leverage_positions,
        refresh_balances,
        deposit_account,
        active_component,
        set_active_component,
        collateral_ratio,
        liquidity_provided,
        tvl,
        icp_price,
        covered_ratio,
        coverable_amount,
        icp_balance,
        eusd_balance,
        is_loading,
        set_is_loading,
    }
}

const stateContext = createContext<StateContext>(null!);

export function ProvideState({ children }) {
    const state = useProvideState();
    return <stateContext.Provider value={state}> {children} </stateContext.Provider>;
}

export const use_local_state = () => {
    return useContext(stateContext);
};