import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array,
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Uint8Array,
  'headers' : Array<[string, string]>,
  'status_code' : number,
}
export type Result = { 'Ok' : bigint } |
  { 'Err' : TransferError };
export type TransferError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'BadBurn' : { 'min_burn_amount' : bigint } } |
  { 'Duplicate' : { 'duplicate_of' : bigint } } |
  { 'BadFee' : { 'expected_fee' : bigint } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export interface _SERVICE {
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'register_user' : ActorMethod<[string], Result>,
  'set_key' : ActorMethod<[string], undefined>,
}
