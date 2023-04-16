import subprocess
import re

class IdentityActor:
    def __init__(self):
        self.name = "identity_"
        self.identities_count = 5
        self.current_identity = 0
    
    def create_identities(self):
        for k in range(self.identities_count):
            name = self.name + str(k)
            command = f'dfx identity new --disable-encryption {name}'
            subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    
    def switch_to_next_identity(self):
        self.current_identity = (self.current_identity + 1) % self.identities_count
        name = self.name + str(self.current_identity)
        command = f'dfx identity use {name}'
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
    
    def get_current_identity_principal(self):
        command = 'dfx identity get-principal'
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')

    def get_all_principals(self):
        users_principals = []
        for k in range(self.identities_count):
            name = self.name + str(k)
            command = f'dfx identity use {name}'
            subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
            command = f'dfx identity get-principal'
            result = subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8').replace('\n','')
            users_principals.append(result)
        return users_principals

    def use_default_identity(self):
        command = 'dfx identity use default'
        return subprocess.run(command, shell=True, stdout=subprocess.PIPE).stdout.decode('utf-8')
