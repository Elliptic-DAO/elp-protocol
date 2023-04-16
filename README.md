
# Elliptic Protocol

 The Elliptic protocol is an innovative decentralized and over-collateralized stablecoin platform that aims to provide a stable and capital-efficient alternative to highly volatile cryptocurrencies. The protocol allows users to exchange their volatile assets from multiple chains into stable assets, providing a stable value in US dollars represented as eUSD. As one of the founding pillars of Decentralized Finance (DeFi), stablecoins play a crucial role in the ecosystem. The Elliptic protocol aims to be at the forefront of innovation by leveraging the cutting-edge technology of the Internet Computer. With its innovative approach, the Elliptic protocol is poised to become the first decentralized multi-chain stablecoin in the blockchain space.

[White Paper](https://t6mee-lqaaa-aaaam-abica-cai.ic0.app/elliptic_dao_whitepaper.pdf)

You can already the beta version of the Elliptic protocol [here](https://t6mee-lqaaa-aaaam-abica-cai.ic0.app/).
- Connect to the Dapp using [Plug](https://chrome.google.com/webstore/detail/plug/cfbfdhimifdmdehjmkdobpcjfefblkjm) or [Bitfinity](https://chrome.google.com/webstore/detail/bitfinity-wallet/jnldfbidonfeldmalbflbmlebbipcnle) wallet.
- You will be granted 50 test ICP (it's not real tokens). 
- You can use them to test the different possibilities the protocol offers.
- If you have any question or want to report a bug join us on [Open Chat](https://oc.app/5gnc5-giaaa-aaaar-alooa-cai/?code=ef8c8f73321e5721).

All the canisters used in the Elliptic procol are defined in the canisters folder. The deployements script are defined in the deployement folder.

## Running the project locally

If you want to test the Elliptic Protocol locally, you need to clone the [Internet Computer repo](https://github.com/dfinity/ic) in the same folder where you cloned the Elliptic Protocol. First you need to compile the canisters of the IC repo.

```bash
cd ic
bazel build ...
```
Then you need to download packages used in the protocol.
```bash
cd elp-protocol
cargo build
npm i
```
Create the .env file.
```bash
touch .env
nano .env
# Define the following env variables:
# REACT_APP_API_KEY="SOMEKEY"
# REACT_APP_NETWORK=ic
# DFX_NETWORK="ic"
```
Deploy the canisters.
```bash
./deployement/local_deploy.sh
```
Launch the React App.
```bash
npm run start
```

## Some Ressources
- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/quickstart/hello10mins)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/build/install-upgrade-remove)
- [Bitfinity Wallet Documentation](https://infinityswap-docs-wallet.web.app/docs/wallet)
- [Deploying ICRC-1 Token](https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/deploy-new-token)
- [White Paper](https://t6mee-lqaaa-aaaam-abica-cai.ic0.app/elliptic_dao_whitepaper.pdf)
  
## Socials
- [Twitter](https://twitter.com/elliptic_dao)
- [Open Chat](https://oc.app/5gnc5-giaaa-aaaar-alooa-cai/?code=ef8c8f73321e5721)

<!-- kill $(lsof -t -i:8080) -->
<!-- killall dfx replica -->