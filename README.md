# Multi-chain Governance: On-Chain Voting with Ethereum and Internet Computer


## Background 
Decentralized voting is a key element of blockchain governance, yet participation in this process on Ethereum is significantly hindered by high gas fees. Alternative solutions like Snapshot have emerged, offering an off-chain voting mechanism to sidestep these fees. However, such solutions introduce considerable challenges, particularly the lack of direct enforceability of votes on the blockchain. This discrepancy can lead to misalignments between governance decisions made off-chain and their actual implementation on-chain.

## Scope
This project implements an ERC-20 based, 100% on-chain voting system enabled by leveraging Ethereum and the Internet Computer blockchain. Its key features include:

* Retrieving ERC-20 voting rights from Ethereum, and integrating them into the multi-chain governance framework.
* Implementing a simple, yet effective, voting application on the Internet Computer, covering frontend and backend.
* Executing the voting process on the ICP blockchain, showcasing the practicability of on-chain voting.
* Triggering actions on Ethereum based on voting outcomes, illustrating a seamless, admin-free integration that ensures that governance decisions are directly reflected on the blockchain.

## Notes
The application leverages the [RPC canister](https://internetcomputer.org/docs/current/developer-docs/multi-chain/ethereum/using-eth/evm-rpc/) for ICP-Ethereum communication and incorporates Ethereum login functionality of the [Sign In with Ethereum (SIWE) project](https://github.com/kristoferlund/ic-siwe/tree/main/packages/ic_siwe_provider) and its [Rust demo](https://github.com/kristoferlund/ic-siwe-react-demo-rust).

## License
This project is distributed under the MIT License, detailed in the LICENSE file.

# Workshop runbook
1. Go to https://sepolia-faucet.pk910.de and start mining some Sepolia ETH
1. Install `dfx`
  - If on Windows: Install [WSL 2](https://learn.microsoft.com/en-us/windows/wsl/basic-commands)
  - Run `sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"` and follow the printed instructions
1. Deploy the voting app locally
  - Start a local ICP node: `dfx start --clean --background` (you can stop it later with `dfx stop`)
    - On Windows: If it doesn't work, deploy to mainnet instead (see below for instructions)
  - Deploy locally: `make deploy-all`. You will see a URL to the frontend in the console
1. Deploy your Solididy smart contract to Sepolia
  - Collect rewards from the faucet
  - Suggested deployment tool: https://remix.ethereum.org/
  - Copy/paste `solidity/contract.sol` into remix
  - Compile
  - Deploy using `WalletConnect` environment to target Sepolia
  - Note the address of the Sepolia contract
1. Configure your contracts/balances
  - Replace `pub const TARGET_CONTRACT` in `src/backend/src/lib.rs` with your smart contract and upgrade the backend (`make deploy-backend`)
  - Figure out the ETH address of your canister: `dfx canister call backend get_eth_address`
  - Transfer some ETH to your canister
  - Set the executor in the Sepolia contract to be your canister
1. Test if the setup works
  - Try creating a proposal and vote on it
  - See if 



# Workshop runbook
1. Go to https://sepolia-faucet.pk910.de and start mining some Sepolia ETH
1. Install `dfx`
  - If on Windows: Install [WSL 2](https://learn.microsoft.com/en-us/windows/wsl/basic-commands)
  - Run `sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"` and follow the printed instructions
1. Deploy the voting app locally
  - Start a local ICP node: `dfx start --clean --background` (you can stop it later with `dfx stop`)
    - On Windows: If it doesn't work, deploy to mainnet instead (see below for instructions)
  - Deploy locally: `make deploy-all`. You will see a URL to the frontend in the console
1. Deploy your Solididy smart contract to Sepolia
  - Collect rewards from the faucet
  - Suggested deployment tool: https://remix.ethereum.org/
  - Copy/paste `solidity/contract.sol` into remix
  - Compile
  - Deploy using `WalletConnect` environment to target Sepolia
  - Note the address of the Sepolia contract
1. Configure your contracts/balances
  - Replace `pub const TARGET_CONTRACT` in `src/backend/src/lib.rs` with your smart contract and upgrade the backend (`make deploy-backend`)
  - Figure out the ETH address of your canister: `dfx canister call backend get_eth_address`
  - Transfer some ETH to your canister
  - Set the executor in the Sepolia contract to be your canister
1. Test if the setup works
  - Try creating a proposal and vote on it
  - See if 

If you want you can also deploy to mainnet.
- If you don't have any cycles (gas) yet, run `dfx wallet --network ic redeem-faucet-coupon <code you receive from the instructor>`
- `DFX_NETWORK=ic make deploy-all`. You will see a URL to the frontend in the console
- The new backend canister will have a different ETH address. Re-configure your smart contract as needed