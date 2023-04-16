import subprocess
import re

#   get_deposit_subaccount : () -> (Account);

#   add_liquidity : (nat64) -> (Result);
#   remove_liquidity : (nat64) -> (Result);
#   claim_liquidity_rewards : () -> (Result);

#   open_leverage_position : (OpenLeveragePositionArg) -> (Result_1);
#   close_leverage_position : (nat64) -> (Result_1);

#   swap : (SwapArg) -> (Result_2);

#   get_events : (GetEventsArg) -> (vec Event) query;
#   get_protocol_status : () -> (ProtocolStatus) query;
#   get_user_data : (principal) -> (UserData) query;
#   http_request : (HttpRequest) -> (HttpResponse) query;
# type Asset = variant { ICP; EUSD };
# type SwapSuccess = record { to_block_index : nat64; from_block_index : nat64 };
# type SwapArg = record { to_asset : Asset; from_asset : Asset; amount : nat64 };


class CoreActor:
    def __init__(self, core_canister_name):
        self.core_canister_name = core_canister_name
    
    def get_deposit_account(self):
        command = f'dfx canister call {self.core_canister_name} get_deposit_subaccount'
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
        match = re.search(r'owner = principal "(.*)";\n.*subaccount = opt blob "(.*)"', result)
        return match.group(1), match.group(2)

    def convert_icp_to_eusd(self, amount):
        command = f'dfx canister call {self.core_canister_name} swap \'(record {{ from_asset= variant {{ ICP }}; to_asset= variant {{EUSD}}; amount={amount} }})\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
    
    def convert_eusd_to_icp(self, amount):
        command = f'dfx canister call {self.core_canister_name} convert_eusd_to_icp \'({amount})\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def add_liquidity(self, amount):
        command = f'dfx canister call {self.core_canister_name} add_liquidity \'({amount})\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def remove_liquidity(self, amount):
        command = f'dfx canister call {self.core_canister_name} remove_liquidity \'({amount})\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def open_leverage_position(self, amount, take_profit, covered_amount):
        command = f'dfx canister call {self.core_canister_name} open_leverage_position \'(record {{ take_profit={take_profit}; amount={amount}; covered_amount={covered_amount} }})\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def close_leverage_position(self, leverage_position):
        command = f'dfx canister call {self.core_canister_name} close_leverage_position \'({leverage_position})\''
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def get_leverage_coverable_amount(self):
        command = f'dfx canister call {self.core_canister_name} get_leverage_coverable_amount'
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def get_first_leverage_position(self):
        command = f'dfx canister call {self.core_canister_name} get_leverage_positions'
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
        return result[18:323].replace('      ','').replace('    ','')


