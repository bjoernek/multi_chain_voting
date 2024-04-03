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
