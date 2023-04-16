from actors.core_actor import CoreActor
from actors.ledger_actor import LedgerActor
from actors.identity_actor import IdentityActor
import re

def get_exchange_rate(base_asset):
    command = f'dfx canister call --wallet rwlgt-iiaaa-aaaaa-aaaaa-cai --with-cycles 10000000000 xrc get_exchange_rate \'(record{{ base_asset=record{{symbol="{base_asset}"; class=variant{{Cryptocurrency}}}}; quote_asset=record{{symbol="USD"; class=variant{{FiatCurrency}}}}}})\''
    result = subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
    # Use regular expressions to extract the decimals and rate
    # decimals_match = re.search(r"decimals = (\d+)", result)
    # rate_match = re.search(r"rate = (\d+)", result)
    
    # if decimals_match and rate_match:
    #     decimals = int(decimals_match.group(1))
    #     rate = int(rate_match.group(1))
    #     return decimals, rate
    # else:
    #     return None
    return result

to_address = "akllx-q5q7v-sgdck-cjd7y-izqql-ck5rp-ee3c7-kzrea-k3fnf-pcuaw-pqe"
amount = 5_000_000_000
covered_amount = 10_000_000_000
icp_canister_name = "icp_ledger"
core_canister_name = "core"
xrc_canister_name = "xrc"
eusd_canister_name = "eUSD"

icp_ledger_actor = LedgerActor(icp_canister_name)
eusd_ledger_actor = LedgerActor(eusd_canister_name)
core_actor = CoreActor(core_canister_name)
identity = IdentityActor()
# identity.create_identities()
# identity.switch_to_next_identity()

def initialize_balance():
    principal_list = identity.get_all_principals()
    print(principal_list)
    identity.use_default_identity()
    icp_ledger_actor.multiple_transfers(principal_list, amount)

initialize_balance()

# Constants
one_icp = 100_000_000
five_icp = 500_000_000
ten_icp = 1_000_000_000
ten_dollars_icp = 1_000_000_000
ten_eusd = 1_000_000_000

identity.switch_to_next_identity()
deposit_account = core_actor.get_deposit_address()
print(deposit_account)
print(icp_ledger_actor.transfer_to_with_subaccount(deposit_account, ten_icp))
print(core_actor.convert_icp_to_eusd(ten_icp))

# deposit_account = core_actor.get_deposit_address()
# eusd_ledger_actor.transfer_to_with_subaccount(deposit_account, ten_eusd)
# print(core_actor.convert_eusd_to_icp(ten_eusd))

# identity.switch_to_next_identity()
# deposit_account = core_actor.get_deposit_address()
# icp_ledger_actor.transfer_to_with_subaccount(deposit_account, ten_icp)
# print(core_actor.add_liquidity(ten_icp))
# print(core_actor.remove_liquidity(one_icp))

# identity.switch_to_next_identity()
# deposit_account = core_actor.get_deposit_address()
# icp_ledger_actor.transfer_to_with_subaccount(deposit_account, ten_icp)
# print(core_actor.add_liquidity(ten_icp))
# print(core_actor.remove_liquidity(one_icp))

# coverable_amount_string = core_actor.get_leverage_coverable_amount()
# match = re.search(r'\((\d+(_\d+)*)\s*:', coverable_amount_string)
# coverable_amount = int(match.group(1).replace('_', ''))

# print(coverable_amount)

# identity.switch_to_next_identity()
# deposit_account = core_actor.get_deposit_address()
# icp_ledger_actor.transfer_to_with_subaccount(deposit_account, ten_icp)
# print(core_actor.open_leverage_position(ten_icp, one_icp, coverable_amount - one_icp))


# deposit_account = core_actor.get_deposit_address()
# print(icp_ledger_actor.transfer_with_subaccount(deposit_account, 10 * amount))
# # for i in range(1):
#     # print(core_actor.add_liquidity(amount))
#     # print(core_actor.convert_icp_to_eusd(amount))
#     # print(core_actor.open_leverage_position(amount, amount, covered_amount))
#     # print(core_actor.get_leverage_position())
# print(core_actor.open_leverage_position(amount, amount, covered_amount))

# res = core_actor.get_first_leverage_position()
# core_actor.close_leverage_position(res)

