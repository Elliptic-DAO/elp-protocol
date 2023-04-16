export const idlFactory = ({ IDL }) => {
    const UpgradeArgs = IDL.Record({});
    const Mode = IDL.Variant({
        'RestrictedTo': IDL.Vec(IDL.Principal),
        'DepositsRestrictedTo': IDL.Vec(IDL.Principal),
        'ReadOnly': IDL.Null,
        'GeneralAvailability': IDL.Null,
        'NoHttpOutCalls': IDL.Null,
    });
    const InitArgs = IDL.Record({
        'eusd_ledger_principal': IDL.Opt(IDL.Principal),
        'min_amount_to_stable': IDL.Opt(IDL.Nat64),
        'xrc_principal': IDL.Opt(IDL.Principal),
        'icp_ledger_principal': IDL.Opt(IDL.Principal),
        'mode': Mode,
        'min_amount_from_stable': IDL.Opt(IDL.Nat64),
        'min_amount_leverage': IDL.Opt(IDL.Nat64),
        'min_amount_liquidity': IDL.Opt(IDL.Nat64),
    });
    const CoreArgs = IDL.Variant({
        'Upgrade': IDL.Opt(UpgradeArgs),
        'Init': InitArgs,
    });
    const TransferError = IDL.Variant({
        'GenericError': IDL.Record({
            'message': IDL.Text,
            'error_code': IDL.Nat,
        }),
        'TemporarilyUnavailable': IDL.Null,
        'BadBurn': IDL.Record({ 'min_burn_amount': IDL.Nat }),
        'Duplicate': IDL.Record({ 'duplicate_of': IDL.Nat }),
        'BadFee': IDL.Record({ 'expected_fee': IDL.Nat }),
        'CreatedInFuture': IDL.Record({ 'ledger_time': IDL.Nat64 }),
        'TooOld': IDL.Null,
        'InsufficientFunds': IDL.Record({ 'balance': IDL.Nat }),
    });
    const LiquidityError = IDL.Variant({
        'NoClaimableReward': IDL.Null,
        'TemporarilyUnavailable': IDL.Text,
        'AlreadyProcessing': IDL.Null,
        'LedgerError': TransferError,
        'NotEnoughLiquidity': IDL.Nat64,
        'NoLiquidityProvided': IDL.Null,
        'AmountTooSmall': IDL.Null,
    });
    const Result = IDL.Variant({ 'Ok': IDL.Nat64, 'Err': LiquidityError });
    const LeveragePositionError = IDL.Variant({
        'PositionNotFound': IDL.Null,
        'TemporarilyUnavailable': IDL.Text,
        'AlreadyProcessing': IDL.Null,
        'LedgerError': TransferError,
        'TooEarlyToClose': IDL.Null,
        'NotEnoughFundsToCover': IDL.Null,
        'IndexNotFound': IDL.Null,
        'CallerNotOwner': IDL.Null,
        'AmountTooSmall': IDL.Null,
    });
    const Result_1 = IDL.Variant({
        'Ok': IDL.Nat64,
        'Err': LeveragePositionError,
    });
    const Account = IDL.Record({
        'owner': IDL.Principal,
        'subaccount': IDL.Opt(IDL.Vec(IDL.Nat8)),
    });
    const GetEventsArg = IDL.Record({
        'start': IDL.Nat64,
        'length': IDL.Nat64,
    });
    const Asset = IDL.Variant({ 'ICP': IDL.Null, 'EUSD': IDL.Null });
    const Swap = IDL.Record({
        'to': Asset,
        'fee': IDL.Nat64,
        'from_amount': IDL.Nat64,
        'from': Asset,
        'rate': IDL.Nat64,
        'from_block_index': IDL.Nat64,
        'timestamp': IDL.Nat64,
        'caller': IDL.Principal,
    });
    const LiquidityType = IDL.Variant({ 'Add': IDL.Null, 'Remove': IDL.Null });
    const Liquidity = IDL.Record({
        'fee': IDL.Nat64,
        'block_index': IDL.Nat64,
        'operation_type': LiquidityType,
        'timestamp': IDL.Nat64,
        'caller': IDL.Principal,
        'amount': IDL.Nat64,
    });
    const SwapSuccess = IDL.Record({
        'to_block_index': IDL.Nat64,
        'from_block_index': IDL.Nat64,
    });
    const IcpPrice = IDL.Record({ 'rate': IDL.Nat64 });
    const LeveragePosition = IDL.Record({
        'fee': IDL.Nat64,
        'take_profit': IDL.Nat64,
        'covered_amount': IDL.Nat64,
        'owner': IDL.Principal,
        'icp_entry_price': IcpPrice,
        'timestamp': IDL.Nat64,
        'deposit_block_index': IDL.Nat64,
        'amount': IDL.Nat64,
    });
    const Event = IDL.Variant({
        'init': InitArgs,
        'swap': Swap,
        'liquidity': Liquidity,
        'upgrade': IDL.Record({}),
        'swap_success': SwapSuccess,
        'claim_liquidity_rewards': IDL.Record({ 'owner': IDL.Principal }),
        'open_leverage_position': LeveragePosition,
        'close_leverage_position': IDL.Record({
            'fee': IDL.Nat64,
            'output_block_index': IDL.Opt(IDL.Nat64),
            'deposit_block_index': IDL.Nat64,
        }),
    });
    const ProtocolStatus = IDL.Record({
        'tvl': IDL.Nat64,
        'collateral_ratio': IDL.Nat64,
        'coverable_amount': IDL.Nat64,
        'icp_price': IDL.Nat64,
        'coverered_ratio': IDL.Nat64,
    });
    const UserData = IDL.Record({
        'liquidity_provided': IDL.Nat64,
        'leverage_positions': IDL.Opt(IDL.Vec(LeveragePosition)),
        'claimable_liquidity_rewards': IDL.Nat64,
    });
    const HttpRequest = IDL.Record({
        'url': IDL.Text,
        'method': IDL.Text,
        'body': IDL.Vec(IDL.Nat8),
        'headers': IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    });
    const HttpResponse = IDL.Record({
        'body': IDL.Vec(IDL.Nat8),
        'headers': IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
        'status_code': IDL.Nat16,
    });
    const OpenLeveragePositionArg = IDL.Record({
        'take_profit': IDL.Nat64,
        'covered_amount': IDL.Nat64,
        'amount': IDL.Nat64,
    });
    const SwapArg = IDL.Record({
        'to_asset': Asset,
        'from_asset': Asset,
        'amount': IDL.Nat64,
    });
    const SwapError = IDL.Variant({
        'NoPriceData': IDL.Null,
        'TemporarilyUnavailable': IDL.Text,
        'AlreadyProcessing': IDL.Null,
        'EUSDLedgerError': TransferError,
        'ICPLedgerError': TransferError,
        'AmountTooSmall': IDL.Null,
    });
    const Result_2 = IDL.Variant({ 'Ok': IDL.Nat64, 'Err': SwapError });
    return IDL.Service({
        'add_liquidity': IDL.Func([IDL.Nat64], [Result], []),
        'claim_liquidity_rewards': IDL.Func([], [Result], []),
        'close_leverage_position': IDL.Func([IDL.Nat64], [Result_1], []),
        'get_deposit_account': IDL.Func([], [Account], []),
        'get_events': IDL.Func([GetEventsArg], [IDL.Vec(Event)], ['query']),
        'get_protocol_status': IDL.Func([], [ProtocolStatus], ['query']),
        'get_user_data': IDL.Func([IDL.Principal], [UserData], ['query']),
        'http_request': IDL.Func([HttpRequest], [HttpResponse], ['query']),
        'open_leverage_position': IDL.Func(
            [OpenLeveragePositionArg],
            [Result_1],
            [],
        ),
        'remove_liquidity': IDL.Func([IDL.Nat64], [Result], []),
        'swap': IDL.Func([SwapArg], [Result_2], []),
    });
};
export const init = ({ IDL }) => {
    const UpgradeArgs = IDL.Record({});
    const Mode = IDL.Variant({
        'RestrictedTo': IDL.Vec(IDL.Principal),
        'DepositsRestrictedTo': IDL.Vec(IDL.Principal),
        'ReadOnly': IDL.Null,
        'GeneralAvailability': IDL.Null,
        'NoHttpOutCalls': IDL.Null,
    });
    const InitArgs = IDL.Record({
        'eusd_ledger_principal': IDL.Opt(IDL.Principal),
        'min_amount_to_stable': IDL.Opt(IDL.Nat64),
        'xrc_principal': IDL.Opt(IDL.Principal),
        'icp_ledger_principal': IDL.Opt(IDL.Principal),
        'mode': Mode,
        'min_amount_from_stable': IDL.Opt(IDL.Nat64),
        'min_amount_leverage': IDL.Opt(IDL.Nat64),
        'min_amount_liquidity': IDL.Opt(IDL.Nat64),
    });
    const CoreArgs = IDL.Variant({
        'Upgrade': IDL.Opt(UpgradeArgs),
        'Init': InitArgs,
    });
    return [CoreArgs];
};
