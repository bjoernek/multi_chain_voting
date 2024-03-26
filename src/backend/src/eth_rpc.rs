use crate::declarations::evm_rpc::*;
use candid::{Nat, Principal};
use ethers_core::abi::ethereum_types::{Address, H160, U256, U64};
use ethers_core::abi::{Contract, FunctionExt, Token, Uint};
use ethers_core::types::Bytes;
use ethers_core::utils::keccak256;
use hex::FromHexError;
use ic_cdk::api::{
    call::{call_with_payment, CallResult},
    management_canister::ecdsa::{
        ecdsa_public_key, sign_with_ecdsa, EcdsaKeyId, EcdsaPublicKeyArgument,
        SignWithEcdsaArgument,
    },
};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

// const CHAIN_ID: u128 = 1337;
const CHAIN_ID: u128 = 11155111;
const GAS: u128 = 80_000;
const MAX_FEE_PER_GAS: u128 = 156_083_066_522_u128;
const MAX_PRIORITY_FEE_PER_GAS: u128 = 3_000_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    id: u64,
    jsonrpc: String,
    method: String,
    params: (EthCallParams, String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EthCallParams {
    to: String,
    data: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonRpcResult {
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: isize,
    message: String,
}

#[macro_export]
macro_rules! include_abi {
    ($file:expr $(,)?) => {{
        match serde_json::from_str::<ethers_core::abi::Contract>(include_str!($file)) {
            Ok(contract) => contract,
            Err(err) => panic!("Error loading ABI contract {:?}: {}", $file, err),
        }
    }};
}

pub fn ecdsa_key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
        name: String::from("test_key_1"),
    }
}

fn next_id() -> u64 {
    thread_local! {
        static NEXT_ID: RefCell<u64> = RefCell::default();
    }
    NEXT_ID.with(|next_id| {
        let mut next_id = next_id.borrow_mut();
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);
        id + 10_512425
    })
}

// Load relevant ABIs (Ethereum equivalent of Candid interfaces)
thread_local! {
    static ETH_CONTRACT: Rc<Contract> = Rc::new(include_abi!("../../../solidity/contract.json"));
}

pub fn parse_address(address_str: &str) -> Result<Address, &'static str> {
    // Remove any leading or trailing whitespace

    // Check if the address string starts with "0x" prefix
    if address_str.starts_with("0x") && address_str.len() == 42 {
        // Try to parse the hexadecimal string into an Address
        if let Ok(address_bytes) = hex::decode(&address_str[2..]) {
            if address_bytes.len() == 20 {
                let mut address = [0u8; 20];
                address.copy_from_slice(&address_bytes);
                return Ok(Address::from(address));
            }
        }
    }

    // If the address format is invalid, return an error
    Err("Invalid Ethereum address format")
}

/// Call an Ethereum smart contract.
pub async fn eth_call(
    contract_address: String,
    abi: &Contract,
    function_name: &str,
    args: &[Token],
    block_number: &str,
) -> Vec<Token> {
    let f = match abi.functions_by_name(function_name).map(|v| &v[..]) {
        Ok([f]) => f,
        Ok(fs) => panic!(
            "Found {} function overloads. Please pass one of the following: {}",
            fs.len(),
            fs.iter()
                .map(|f| format!("{:?}", f.abi_signature()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Err(_) => abi
            .functions()
            .find(|f| function_name == f.abi_signature())
            .expect("Function not found"),
    };
    let data = f
        .encode_input(args)
        .expect("Error while encoding input args");
    let json_rpc_payload = serde_json::to_string(&JsonRpcRequest {
        id: next_id(),
        jsonrpc: "2.0".to_string(),
        method: "eth_call".to_string(),
        params: (
            EthCallParams {
                to: contract_address,
                data: to_hex(&data),
            },
            block_number.to_string(),
        ),
    })
    .expect("Error while encoding JSON-RPC request");

    let res: CallResult<(RequestResult,)> = call_with_payment(
        crate::declarations::evm_rpc::evm_rpc.0,
        "request",
        (
            RpcService::EthSepolia(EthSepoliaService::BlockPi),
            json_rpc_payload,
            2048_u64,
        ),
        2_000_000_000,
    )
    .await;

    match res {
        Ok((RequestResult::Ok(ok),)) => {
            let json: JsonRpcResult =
                serde_json::from_str(&ok).expect("JSON was not well-formatted");
            let result = from_hex(&json.result.expect("Unexpected JSON response")).unwrap();
            f.decode_output(&result).expect("Error decoding output")
        }
        err => panic!("Response error: {err:?}"),
    }
}

/// Submit an ETH TX.
pub async fn eth_transaction(
    contract_address: String,
    abi: &Contract,
    function_name: &str,
    args: &[Token],
) -> String {
    let f = match abi.functions_by_name(function_name).map(|v| &v[..]) {
        Ok([f]) => f,
        Ok(fs) => panic!(
            "Found {} function overloads. Please pass one of the following: {}",
            fs.len(),
            fs.iter()
                .map(|f| format!("{:?}", f.abi_signature()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Err(_) => abi
            .functions()
            .find(|f| function_name == f.abi_signature())
            .expect("Function not found"),
    };
    let data = f
        .encode_input(args)
        .expect("Error while encoding input args");
    let signed_data = sign_transaction(SignRequest {
        chain_id: CHAIN_ID.into(),
        to: contract_address,
        gas: GAS.into(),
        max_fee_per_gas: MAX_FEE_PER_GAS.into(),
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS.into(),
        value: 0_u8.into(),
        nonce: next_id().into(),
        data: Some(data.into()),
    })
    .await;

    let (res,): (MultiSendRawTransactionResult,) = call_with_payment(
        crate::declarations::evm_rpc::evm_rpc.0,
        "eth_sendRawTransaction",
        (
            RpcServices::EthSepolia(Some(vec![
                EthSepoliaService::PublicNode,
                EthSepoliaService::BlockPi,
                EthSepoliaService::Ankr,
            ])),
            None::<RpcConfig>,
            signed_data.clone(),
        ),
        2_000_000_000,
    )
    .await
    .unwrap();

    match res {
        MultiSendRawTransactionResult::Consistent(SendRawTransactionResult::Ok(
            SendRawTransactionStatus::Ok,
        )) => "OK".into(),
        other => format!("call: {signed_data}, error: {:?}", other),
        // Ok((RequestResult::Ok(ok),)) => {
        //     let json: JsonRpcResult =
        //         serde_json::from_str(&ok).expect("JSON was not well-formatted");
        //     let result = from_hex(&json.result.expect("Unexpected JSON response")).unwrap();
        //     f.decode_output(&result).expect("Error decoding output")
        // }
        // err => panic!("Response error: {err:?}"),
    }
}

fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

pub fn from_hex(data: &str) -> Result<Vec<u8>, FromHexError> {
    hex::decode(&data[2..])
}

#[derive(Debug)]
pub struct SignRequest {
    pub chain_id: Nat,
    pub to: String,
    pub gas: Nat,
    pub max_fee_per_gas: Nat,
    pub max_priority_fee_per_gas: Nat,
    /// ETH to send
    pub value: Nat,
    pub nonce: Nat,
    pub data: Option<Bytes>,
}

/// Computes a signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
pub async fn sign_transaction(req: SignRequest) -> String {
    use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
    use ethers_core::types::Signature;

    const EIP1559_TX_ID: u8 = 2;

    let tx = Eip1559TransactionRequest {
        chain_id: Some(nat_to_u64(&req.chain_id)),
        from: None,
        to: Some(
            Address::from_str(&req.to)
                .expect("failed to parse the destination address")
                .into(),
        ),
        gas: Some(nat_to_u256(&req.gas)),
        value: Some(nat_to_u256(&req.value)),
        nonce: Some(nat_to_u256(&req.nonce)),
        data: req.data,
        access_list: Default::default(),
        max_priority_fee_per_gas: Some(nat_to_u256(&req.max_priority_fee_per_gas)),
        max_fee_per_gas: Some(nat_to_u256(&req.max_fee_per_gas)),
    };

    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);

    let txhash = keccak256(&unsigned_tx_bytes);

    let (pubkey, signature) = pubkey_and_signature(txhash.to_vec()).await;

    let signature = Signature {
        v: y_parity(&txhash, &signature, &pubkey),
        r: U256::from_big_endian(&signature[0..32]),
        s: U256::from_big_endian(&signature[32..64]),
    };

    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);

    format!("0x{}", hex::encode(&signed_tx_bytes))
}

/// Computes the parity bit allowing to recover the public key from the signature.
fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> u64 {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).expect("failed to parse the pubkey");
    let signature = Signature::try_from(sig).unwrap();
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).unwrap();
        let recovered_key = VerifyingKey::recover_from_prehash(prehash, &signature, recid)
            .expect("failed to recover key");
        if recovered_key == orig_key {
            return parity as u64;
        }
    }

    panic!(
        "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
        hex::encode(sig),
        hex::encode(pubkey)
    )
}

/// Returns the public key and a message signature for the specified principal.
async fn pubkey_and_signature(message_hash: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    // Fetch the pubkey and the signature concurrently to reduce latency.
    let (pubkey, response) = futures::join!(
        ecdsa_public_key(EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: vec![],
            key_id: ecdsa_key_id()
        }),
        sign_with_ecdsa(SignWithEcdsaArgument {
            message_hash,
            derivation_path: vec![],
            key_id: ecdsa_key_id(),
        })
    );
    (
        pubkey.unwrap().0.public_key,
        response.expect("failed to sign the message").0.signature,
    )
}
fn nat_to_u256(n: &Nat) -> U256 {
    let be_bytes = n.0.to_bytes_be();
    U256::from_big_endian(&be_bytes)
}

fn nat_to_u64(n: &Nat) -> U64 {
    let be_bytes = n.0.to_bytes_be();
    U64::from_big_endian(&be_bytes)
}

fn decode_hex(hex: &str) -> Bytes {
    Bytes::from(hex::decode(hex.trim_start_matches("0x")).expect("failed to decode hex"))
}

pub async fn rpc_request_with_cycles(
    cycles: u64,
    arg1: String,
    max_response_bytes: u64,
) -> CallResult<(RequestResult,)> {
    call_with_payment(
        crate::declarations::evm_rpc::evm_rpc.0,
        "request",
        (
            RpcService::EthSepolia(EthSepoliaService::Alchemy),
            arg1,
            max_response_bytes,
        ),
        cycles,
    )
    .await
}

/// returns latest block number in `U256` and hex encoded form
pub async fn block_number() -> (U256, String) {
    let RequestResult::Ok(response) = rpc_request_with_cycles(
        1_000_000_000,
        "{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[]}".into(),
        2000,
    )
    .await
    .expect("RPC failed")
    .0
    else {
        panic!("oops")
    };
    let json: JsonRpcResult = serde_json::from_str(&response).expect("JSON was not well-formatted");
    if let Some(err) = json.error {
        panic!("JSON-RPC error code {}: {}", err.code, err.message);
    }
    let hex_result = json.result.expect("Unexpected JSON response");
    let result = from_hex(&hex_result).unwrap();

    (U256::from_big_endian(&result), hex_result)
}

pub async fn balance_of(user: &str, block_number: &str) -> Uint {
    let Token::Uint(balance) = eth_call(
        super::TARGET_CONTRACT.into(),
        &ETH_CONTRACT.with(Rc::clone),
        "balanceOf",
        &[Token::Address(parse_address(user).unwrap())],
        block_number,
    )
    .await
    .get(0)
    .unwrap()
    .clone() else {
        panic!("oops")
    };
    balance
}

pub async fn transfer_to(to: &str, amount: u128) -> String {
    eth_transaction(
        super::TARGET_CONTRACT.into(),
        &ETH_CONTRACT.with(Rc::clone),
        "transfer",
        &[
            Token::Address(parse_address(to).unwrap()),
            Token::Uint(amount.into()),
        ],
    )
    .await
}

pub async fn get_self_eth_address() -> String {
    let (pubkey,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: vec![],
        key_id: ecdsa_key_id(),
    })
    .await
    .unwrap();

    let key = PublicKey::from_sec1_bytes(&pubkey.public_key)
        .expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);

    let hash = keccak256(&point_bytes[1..]);

    ethers_core::utils::to_checksum(&Address::from_slice(&hash[12..32]), None)
}
