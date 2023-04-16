import subprocess
import re

class LedgerActor:
    def __init__(self, canister_name):
        self.canister_name = canister_name

    def transfer_to(self, to_address, amount):
        command = f'dfx canister call {self.canister_name} icrc1_transfer \'(record {{\n  to = record {{owner = principal "{to_address}"}};\n  amount={amount}\n}})\''
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
        return result

    def transfer_to_with_subaccount(self, to, amount):
        command = f'dfx canister call {self.canister_name} icrc1_transfer \'(record {{\n  to = record {{\n    owner = principal "{to[0]}";\n    subaccount = opt blob "{to[1]}";\n  }};\n  amount={amount}\n}})\''
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
        return result

    def get_balance(self, address):
        command = f'dfx canister call {self.canister_name} icrc1_balance_of \'(record {{ owner=principal "{address}" }},)\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def multiple_transfers(self, principal_list, amount):
        for to_address in principal_list:
            command = f'dfx canister call {self.canister_name} icrc1_transfer \'(record {{\n  to = record {{owner = principal "{to_address}"}};\n  amount={amount}\n}})\''
            print(subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8'))
