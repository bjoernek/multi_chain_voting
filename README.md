# Multi-chain Governance: On-Chain Voting with Ethereum and Internet Computer


## Background 
Decentralized voting is a key element of blockchain governance, yet participation in this process on Ethereum is significantly hindered by high gas fees. Alternative solutions like Snapshot have emerged, offering an off-chain voting mechanism to sidestep these fees. However, such solutions introduce considerable challenges, particularly the lack of direct enforceability of votes on the blockchain. This discrepancy can lead to misalignments between governance decisions made off-chain and their actual implementation on-chain.

## Scope
This project implements an Ethereum token based, 100% on-chain voting system enabled by leveraging Ethereum and the Internet Computer blockchain. Its key features include:

* Retrieving voting rights from Ethereum (Ethereum Sepolia or ERC-20 based), and integrating them into the multi-chain governance framework.
* Implementing a simple, yet effective, voting application on the Internet Computer, covering frontend and backend.
* Executing the voting process on the ICP blockchain, showcasing the practicability of on-chain voting.
* Triggering actions on Ethereum based on voting outcomes, illustrating a seamless, admin-free integration that ensures that governance decisions are directly reflected on the blockchain.

## Notes
This is only a proof of concept and aims for simplicity of code, not security.

The application leverages the [RPC canister](https://internetcomputer.org/docs/current/developer-docs/multi-chain/ethereum/using-eth/evm-rpc/) for ICP-Ethereum communication and incorporates Ethereum login functionality of the [Sign In with Ethereum (SIWE) project](https://github.com/kristoferlund/ic-siwe/tree/main/packages/ic_siwe_provider) and its [Rust demo](https://github.com/kristoferlund/ic-siwe-react-demo-rust).

## License
This project is distributed under the MIT License, detailed in the LICENSE file.

# Runbook
0. Clone this repo and `cd` into it: `git clone git@github.com:bjoernek/multi_chain_voting.git && cd multi_chain_voting`
1. Go to https://sepolia-faucet.pk910.de and start mining some SepoliaETH
2. Install `dfx`
  - If on Windows: Install [WSL 2](https://learn.microsoft.com/en-us/windows/wsl/basic-commands)
  - Run `sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"` and follow the printed instructions
  - Run `dfxvm update`
3. Deploy the voting app locally
  - Start a local ICP node: `dfx start --clean --background` (you can stop it later with `dfx stop`)
    - On Windows: If it doesn't work, deploy to mainnet instead (see below for instructions)
  - Deploy locally: `make deploy-all`. You will see a URL to the frontend in the console
4. Claim SepoliaETH from the faucet
5. Try the dapp
  - Navigate to the frontend URL displayed in the terminal
  - Log in, create a proposal, vote on it. You should have voting power equal to the amount of SepoliaETH you have
6. If you want your backend to write the results to Sepolia
  - Send your backend some SepoliaETH. You can find its address with `dfx canister call backend get_eth_address`
  - Call `dfx canister call backend execute_proposal '(<proposal id>)'`
    - The response contains a TX id you can look up on e.g. Etherscan
  - You can also deploy your own contract on Sepolia
    - Suggested deployment tool: https://remix.ethereum.org/
    - Copy/paste `solidity/contract.sol` into remix
    - Compile
    - Deploy using `WalletConnect` environment to target Sepolia
    - Note the address of the Sepolia contract
    - Replace `pub const TARGET_CONTRACT` in `src/backend/src/lib.rs` with your newly deployed Sepolia contract
    - Redeploy the backend: `make deploy-backend`

If you want you can also deploy to ICP mainnet.
- If you don't have any cycles (gas) yet, run `dfx wallet --network ic redeem-faucet-coupon <code>`
- `DFX_NETWORK=ic make deploy-all`. You will see a URL to the frontend in the console
- The new backend canister will have a different ETH address. Send it some SepoliaETH if you want it to be able to write to Sepolia

# Extension: Proposal type for upgrading an ICP-hosted Ethereum dapp frontend

This demo extension implements upgrading an ICP-hosted Ethereum frontend with a vote of Ethereum users on ICP. This allows to have a DAO on ICP governed by Ethereum users, using the voting power based on token holdings on the Ethereum network, and upgrading an ICP-hosted Ethereum frontend.

The example frontend is hosted as a separate project (??) and comprises a dummy UI for UniSwap that is hosted as an ICP canister. This canister allows for the `Prepare` and `Commit` operations. `Prepare` uploads a new frontend to the canister, while not making it accessible yet for users. `Commit` commits the uploaded changes from the `Prepare` to the canister and makes them available as the current version of the frontend.

The demo proceeds as follows:
* An entity with the `Prepare` permission uploads an updated frontend to the canister using the `dfx deploy asset_frontend --by-proposal --network ic` command.
  * The GitHub repository for the demo frontend canister contains two branches with two visually very distinct user interfaces, but only a small change set to allow for fast uploading. You can use the following commands to check out the versions for the demo:
    * `git checkout pre-upgrade-frontend` to checkout a boring-looking frontend to upload to the canister intially. This upload can be done using `dfx deploy asset_frontend` in the demo. Note that in a real-world deployment the uploader would not have commit rights, but for simplifying the demo they can have.
    * `git checkout post-upgrade-frontend` to checkout the new frontend to upload to the canister with the `...--by-proposal` command. This command output `batch_id` and `evidence` values which are used as input to the voting process. Anyone can validate the correctness of the update by having the previous version and the version after the `Prepare` of the canister and the `batch_id` and `evidence` by re-running the `deploy` operation locally using `dfx deploy asset_frontend --by-proposal`.
* The voting demo is used to create and vote on a proposal of type `Upgrade Ethereum Frontend` using the `batch_id` and `evidence` as the parameters to vote on.
* Once the vote concludes successfully, the governance canister calls the `commit_proposed_batch` method of the asset canister hosting the frontend.

## Extension runbook

The steps explained in the runbook above apply to this extension part of the demo as well. However, additional steps are required in order to show the extension.

* Deploy the Ethereum frontend on an asset canister on ICP as explained in (FIXME link).
* Assign permissions to the frontend canister as follows:
  * `Prepare` permission to the principal you use for uploading the updated UI. This is probably the principal you used to deploy the canister and is set automatically.
  * `Commit` permission to the principal of the backend canister of the voting demo. This backend needs to have the rights to call the `commit_proposed_batch` method of the frontend canister.

  