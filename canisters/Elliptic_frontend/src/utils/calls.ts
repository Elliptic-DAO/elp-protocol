import { idlFactory as core_idl } from "./interfaces/core/core_idl";
import { idlFactory as icrc1_idl } from "./interfaces/icrc1/icrc1_idl";
import { TransferArg, Account } from './interfaces/icrc1/icrc1';
import { OpenLeveragePositionArg, SwapArg } from './interfaces/core/core';
import { CORE_PRINCIPAL } from "./auth";

export function icrc1_transfer(destination_account: Account, amount_to_transfer: bigint, ledger_principal: string) {
    return {
        idl: icrc1_idl,
        canisterId: ledger_principal,
        methodName: 'icrc1_transfer',
        args: [{
            to: destination_account,
            fee: [],
            memo: [],
            from_subaccount: [],
            created_at_time: [],
            amount: amount_to_transfer
        } as TransferArg],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Transfer ICP error:', res);
        },
    };
}


export function convert_icp_to_eusd(amount_to_swap: bigint) {
    return {
        idl: core_idl,
        canisterId: CORE_PRINCIPAL,
        methodName: 'swap',
        args: [{
            amount: amount_to_swap,
            from_asset: { 'ICP': null },
            to_asset: { 'EUSD': null }
        } as SwapArg],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Swap ICP to eUSD error:', res);
        },
    };
}

export function convert_eusd_to_icp(amount_to_swap: bigint) {
    return {
        idl: core_idl,
        canisterId: CORE_PRINCIPAL,
        methodName: 'swap',
        args: [{
            amount: amount_to_swap,
            from_asset: { 'EUSD': null },
            to_asset: { 'ICP': null }
        } as SwapArg],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Swap eUSD to ICP error:', res);
        },
    };
}

export function open_leverage_position(take_profit: bigint, covered_amount: bigint, amount: bigint) {
    return {
        idl: core_idl,
        canisterId: CORE_PRINCIPAL,
        methodName: 'open_leverage_position',
        args: [{
            take_profit,
            covered_amount,
            amount
        } as OpenLeveragePositionArg],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Open Leverage Position error:', res);
        },
    }
}

export function close_leverage_position(deposit_block_index: bigint) {
    return {
        idl: core_idl,
        canisterId: CORE_PRINCIPAL,
        methodName: 'close_leverage_position',
        args: [deposit_block_index],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Close Leverage Position error:', res);
        },
    }
}

enum Operation {
    Add,
    Remove
}

export function liquidity_operation(operation_type: Operation, amount: bigint) {
    let method_name = 'remove_liquidity';
    if (operation_type === Operation.Add) {
        method_name = 'add_liquidity'
    }
    return {
        idl: core_idl,
        canisterId: CORE_PRINCIPAL,
        methodName: method_name,
        args: [amount],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Liquidity Operation error:', res);
        },
    }
}

export function claim_liquidity_rewards() {
    return {
        idl: core_idl,
        canisterId: CORE_PRINCIPAL,
        methodName: 'claim_liquidity_rewards',
        args: [],
        onSuccess: async (res) => {
            console.log(res);
        },
        onFail: (res) => {
            console.log('Claim liquidity error:', res);
        },
    }
}