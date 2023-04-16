import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Account {
    'owner': Principal,
    'subaccount': [] | [Uint8Array],
}
export type Asset = { 'ICP': null } |
{ 'EUSD': null };
export type CoreArgs = { 'Upgrade': [] | [UpgradeArgs] } |
{ 'Init': InitArgs };
export type Event = { 'init': InitArgs } |
{ 'swap': Swap } |
{ 'liquidity': Liquidity } |
{ 'upgrade': {} } |
{ 'swap_success': SwapSuccess } |
{ 'claim_liquidity_rewards': { 'owner': Principal } } |
{ 'open_leverage_position': LeveragePosition } |
{
    'close_leverage_position': {
        'fee': bigint,
        'output_block_index': [] | [bigint],
        'deposit_block_index': bigint,
    }
};
export interface GetEventsArg { 'start': bigint, 'length': bigint }
export interface HttpRequest {
    'url': string,
    'method': string,
    'body': Uint8Array,
    'headers': Array<[string, string]>,
}
export interface HttpResponse {
    'body': Uint8Array,
    'headers': Array<[string, string]>,
    'status_code': number,
}
export interface IcpPrice { 'rate': bigint }
export interface InitArgs {
    'eusd_ledger_principal': [] | [Principal],
    'min_amount_to_stable': [] | [bigint],
    'xrc_principal': [] | [Principal],
    'icp_ledger_principal': [] | [Principal],
    'mode': Mode,
    'min_amount_from_stable': [] | [bigint],
    'min_amount_leverage': [] | [bigint],
    'min_amount_liquidity': [] | [bigint],
}
export interface LeveragePosition {
    'fee': bigint,
    'take_profit': bigint,
    'covered_amount': bigint,
    'owner': Principal,
    'icp_entry_price': IcpPrice,
    'timestamp': bigint,
    'deposit_block_index': bigint,
    'amount': bigint,
}
export type LeveragePositionError = { 'PositionNotFound': null } |
{ 'TemporarilyUnavailable': string } |
{ 'AlreadyProcessing': null } |
{ 'LedgerError': TransferError } |
{ 'TooEarlyToClose': null } |
{ 'NotEnoughFundsToCover': null } |
{ 'IndexNotFound': null } |
{ 'CallerNotOwner': null } |
{ 'AmountTooSmall': null };
export interface Liquidity {
    'fee': bigint,
    'block_index': bigint,
    'operation_type': LiquidityType,
    'timestamp': bigint,
    'caller': Principal,
    'amount': bigint,
}
export type LiquidityError = { 'NoClaimableReward': null } |
{ 'TemporarilyUnavailable': string } |
{ 'AlreadyProcessing': null } |
{ 'LedgerError': TransferError } |
{ 'NotEnoughLiquidity': bigint } |
{ 'NoLiquidityProvided': null } |
{ 'AmountTooSmall': null };
export type LiquidityType = { 'Add': null } |
{ 'Remove': null };
export type Mode = { 'RestrictedTo': Array<Principal> } |
{ 'DepositsRestrictedTo': Array<Principal> } |
{ 'ReadOnly': null } |
{ 'GeneralAvailability': null } |
{ 'NoHttpOutCalls': null };
export interface OpenLeveragePositionArg {
    'take_profit': bigint,
    'covered_amount': bigint,
    'amount': bigint,
}
export interface ProtocolStatus {
    'tvl': bigint,
    'collateral_ratio': bigint,
    'coverable_amount': bigint,
    'icp_price': bigint,
    'coverered_ratio': bigint,
}
export type Result = { 'Ok': bigint } |
{ 'Err': LiquidityError };
export type Result_1 = { 'Ok': bigint } |
{ 'Err': LeveragePositionError };
export type Result_2 = { 'Ok': bigint } |
{ 'Err': SwapError };
export interface Swap {
    'to': Asset,
    'fee': bigint,
    'from_amount': bigint,
    'from': Asset,
    'rate': bigint,
    'from_block_index': bigint,
    'timestamp': bigint,
    'caller': Principal,
}
export interface SwapArg {
    'to_asset': Asset,
    'from_asset': Asset,
    'amount': bigint,
}
export type SwapError = { 'NoPriceData': null } |
{ 'TemporarilyUnavailable': string } |
{ 'AlreadyProcessing': null } |
{ 'EUSDLedgerError': TransferError } |
{ 'ICPLedgerError': TransferError } |
{ 'AmountTooSmall': null };
export interface SwapSuccess {
    'to_block_index': bigint,
    'from_block_index': bigint,
}
export type TransferError = {
    'GenericError': { 'message': string, 'error_code': bigint }
} |
{ 'TemporarilyUnavailable': null } |
{ 'BadBurn': { 'min_burn_amount': bigint } } |
{ 'Duplicate': { 'duplicate_of': bigint } } |
{ 'BadFee': { 'expected_fee': bigint } } |
{ 'CreatedInFuture': { 'ledger_time': bigint } } |
{ 'TooOld': null } |
{ 'InsufficientFunds': { 'balance': bigint } };
export type UpgradeArgs = {};
export interface UserData {
    'liquidity_provided': bigint,
    'leverage_positions': [] | [Array<LeveragePosition>],
    'claimable_liquidity_rewards': bigint,
}
export interface _SERVICE {
    'add_liquidity': ActorMethod<[bigint], Result>,
    'claim_liquidity_rewards': ActorMethod<[], Result>,
    'close_leverage_position': ActorMethod<[bigint], Result_1>,
    'get_deposit_account': ActorMethod<[], Account>,
    'get_events': ActorMethod<[GetEventsArg], Array<Event>>,
    'get_protocol_status': ActorMethod<[], ProtocolStatus>,
    'get_user_data': ActorMethod<[Principal], UserData>,
    'http_request': ActorMethod<[HttpRequest], HttpResponse>,
    'open_leverage_position': ActorMethod<[OpenLeveragePositionArg], Result_1>,
    'remove_liquidity': ActorMethod<[bigint], Result>,
    'swap': ActorMethod<[SwapArg], Result_2>,
}
