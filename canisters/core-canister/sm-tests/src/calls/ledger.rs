use candid::{Decode, Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use ic_ledger_core::block::BlockIndex;
use ic_state_machine_tests::{CanisterId, StateMachine};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use num_traits::ToPrimitive;

pub fn send_transfer(
    env: &StateMachine,
    ledger: CanisterId,
    from: Principal,
    arg: &TransferArg,
) -> Result<BlockIndex, TransferError> {
    Decode!(
        &env.execute_ingress_as(
            PrincipalId(from),
            ledger,
            "icrc1_transfer",
            Encode!(arg)
            .unwrap()
        )
        .expect("failed to transfer funds")
        .bytes(),
        Result<Nat, TransferError>
    )
    .expect("failed to decode transfer response")
    .map(|n| n.0.to_u64().unwrap())
}

pub fn get_balance_of(env: &StateMachine, ledger: CanisterId, arg: &Account) -> Nat {
    Decode!(
        &env.query(ledger, "icrc1_balance_of", Encode!(arg).unwrap())
            .expect("failed to transfer funds")
            .bytes(),
        Nat
    )
    .expect("failed to decode transfer response")
}
