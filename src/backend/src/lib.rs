mod declarations;
mod eth_rpc;
mod service;
mod user_profile;

use crate::eth_rpc::eth_balance_of;
use candid::{CandidType, Deserialize, Nat};
use eth_rpc::{eth_transaction, get_self_eth_address, latest_block_number, rpc_request_with_cycles};
use ethers_core::abi::{Contract, Token};
use ic_cdk_macros::export_candid;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use serde_bytes::ByteBuf;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use user_profile::UserProfile;

use ic_cdk::api::{caller, time};
use ic_cdk::{init, post_upgrade, println, query, update};
use std::collections::HashMap;
use std::time::Duration;

use ic_cdk::api::{
    call::{call_with_payment, CallResult},
    management_canister::ecdsa::{
        ecdsa_public_key, sign_with_ecdsa, EcdsaKeyId, EcdsaPublicKeyArgument,
        SignWithEcdsaArgument,
    },
};

use crate::declarations::evm_rpc::*;
use crate::declarations::ethereum_frontend::*;

use candid::Principal;
use hex::decode;

pub const PROPOSAL_TYPE_MOTION: &str = "Motion";
pub const PROPOSAL_TYPE_UPGRADE_ETH_FE: &str = "UpgradeEthFE";

pub const TARGET_CONTRACT: &str = "0x2036081922cf3124E9f13b3a3a4bE55410C80D95";
// Load relevant ABIs (Ethereum equivalent of Candid interfaces)
thread_local! {
    pub static ETH_CONTRACT: Rc<Contract> = Rc::new(include_abi!("../../../solidity/contract.json"));
}

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static USER_PROFILES: RefCell<StableBTreeMap<String, UserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static VOTES: RefCell<HashMap<u64, HashMap<String, bool>>> = RefCell::new(HashMap::new());
    static PROPOSALS: RefCell<Vec<Proposal>> = RefCell::new(Vec::new());
    static ECDSA_KEY: RefCell<String> = RefCell::new(String::default());
}

// Duration for periodic checks of proposals
const TIMER_INTERVAL: Duration = Duration::from_secs(60);

#[derive(CandidType, Deserialize, Clone, Debug)]
struct Proposal {
    id: u64,
    title: String,
    description: String,
    proposal_type: String,
    batch_id: Option<u64>,
    evidence: Option<Vec<u8>>,
    submitter: String,
    submitter_eth_address: String,
    proposal_start_timestamp: u64,
    proposal_end_timestamp: u64,
    is_open: bool,
    is_executed: bool,
    yes_votes: Nat,
    no_votes: Nat,
    block_height: String,
    eth_transaction_hash: Option<String>,
}

#[update]
async fn submit_proposal(
    title: String,
    description: String,
    proposal_type: String,
    batch_id: Option<u64>,
    evidence: Option<String>,
    duration_seconds: u64,
) -> u64 {
    let submitter = caller().to_text();
    // Initialize submitter_eth_address as an empty string or an appropriate default value
    let mut submitter_eth_address: String = "".to_string();

    let proposal_start_timestamp = time();
    let duration_in_nanoseconds = duration_seconds * 1_000_000_000;
    let proposal_end_timestamp = proposal_start_timestamp + duration_in_nanoseconds;
    println!("Proposal start timestamp: {}", proposal_start_timestamp);
    println!(
        "Computed proposal end timestamp: {}",
        proposal_end_timestamp
    );

    // Attempt to get the address asynchronously
    match service::save_my_profile::get_address().await {
        Ok(address) => {
            println!("Address: {}", address);
            // Store the address in submitter_eth_address if successful
            submitter_eth_address = address;
        }
        Err(e) => {
            println!("Error retrieving address: {}", e);
        }
    }

    // handle Ethereum FE upgrade fields
    println!("batch_id = {:?}, evidence = {:?}", batch_id, evidence);

    let evidence_bytes = match evidence {
        Some(val) => {
            match decode(val) {
                Ok(r) => Some(r),
                Err(e) => { println!("Error with evidence: Decoding of evidence string failed. Be sure you have not copied the trailing \".\" from the command line with the evidence when updating the frontend canister. {}", e); panic!("Error with evidence: Decoding of evidence string failed. Be sure you have not copied the trailing \".\" from the command line with the evidence when updating the frontend canister."); },
            }
        },
        None => None,
    };
        
    // consistency check for attributes of the proposal, depending on proposal type
    if proposal_type == PROPOSAL_TYPE_MOTION {
        if batch_id != None || evidence_bytes != None {
            panic!("Error: batch_id or evidence is defined, proposal submission failed. Please try again with correctly provided information.");
        }
    } else if proposal_type == PROPOSAL_TYPE_UPGRADE_ETH_FE {
        if batch_id == None || evidence_bytes == None {
            panic!("Error: batch_id or evidence is None, proposal submission failed. Please try again with correctly provided information.");
        }
    }
    
    //let evidence_bytes = Some(decode(evidence.unwrap()).unwrap());

    let (_, block_height) = latest_block_number().await;
    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        let new_id = proposals.len() as u64 + 1; // Simple ID generation
        let proposal = Proposal {
            id: new_id,
            title,
            description,
            proposal_type,
            batch_id, // optional field for Upgrading Ethereum FE proposal
            evidence: evidence_bytes, // optional field for Upgrading Ethereum FE proposal
            submitter,
            submitter_eth_address, // This will be empty or contain the address from get_address()
            proposal_start_timestamp,
            proposal_end_timestamp,
            is_open: true,
            is_executed: false,
            yes_votes: 0_usize.into(), // No votes yet
            no_votes: 0_usize.into(),  // No votes yet
            block_height,
            eth_transaction_hash: None,
        };

        proposals.push(proposal);
        new_id // Returning the ID of the new proposal
    })
}

#[query]
fn get_proposals() -> Vec<Proposal> {
    PROPOSALS.with(|proposals_ref| proposals_ref.borrow().clone())
}

#[update]
async fn vote_on_proposal(proposal_id: u64, vote: bool) -> Result<(), String> {
    let voter_principal = caller().to_text();
    println!(
        "Received vote: {}, from principal: {}, for proposal: {}",
        vote, voter_principal, proposal_id
    );

    // Attempt to find the proposal and check its status
    let proposal_check = PROPOSALS.with(|proposals| {
        let proposals = proposals.borrow();
        proposals.iter().find(|&p| p.id == proposal_id).map_or_else(
            || Err("Proposal not found".to_string()),
            |proposal| {
                // Check is_open flag and the time stamp, as the periodic check for proposal status might be outstanding
                if !proposal.is_open || proposal.proposal_end_timestamp < time() {
                    Err("Proposal is already closed".to_string())
                } else {
                    Ok((proposal.block_height.clone(), proposal.is_open))
                }
            },
        )
    })?;
    let block_number = proposal_check.0;

    let voter = match service::save_my_profile::get_address().await {
        Ok(address) => address,
        Err(e) => {
            ic_cdk::trap(&format!("Error retrieving address: {}", e));
        }
    };

    // Ensure each principal votes only once per proposal.
    let already_voted = VOTES.with(|votes| {
        let mut votes = votes.borrow_mut();
        let proposal_votes = votes.entry(proposal_id).or_insert_with(HashMap::new);

        if proposal_votes.contains_key(&voter) {
            println!(
                "Voter {} has already voted on proposal: {}",
                voter, proposal_id
            );
            true
        } else {
            // Record the new vote.
            proposal_votes.insert(voter.clone(), vote);
            println!(
                "Vote: {} recorded for voter {} on proposal: {}",
                vote, voter, proposal_id
            );
            false
        }
    });

    if already_voted {
        return Err("You have already voted on this proposal".to_string());
    }

    let voting_power = eth_balance_of(&voter, &block_number).await;

    // Update the proposal's vote tally
    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        if let Some(proposal) = proposals.iter_mut().find(|p| p.id == proposal_id) {
            if vote {
                println!(
                    "Incremented yes votes by {} for proposal ID: {}",
                    voting_power, proposal_id
                );
                proposal.yes_votes += voting_power;
            } else {
                println!(
                    "Incremented no votes by {} for proposal ID: {}",
                    voting_power, proposal_id
                );
                proposal.no_votes += voting_power;
            }
        }
    });

    Ok(())
}

#[update]
async fn execute_proposal(proposal_id: u64) -> Result<String, String> {
    
    let mut proposal_type = None;
    let mut batch_id = None;
    let mut evidence = None;

    let eth_tx_summary = PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        let proposal = proposals.iter_mut().find(|p| p.id == proposal_id)
            .ok_or_else(|| format!("Proposal {proposal_id} not found."))?;
        if proposal.is_executed {
            return Err(format!("Proposal {proposal_id} already executed"));
        }

        proposal.is_executed = true;

        proposal_type = Some(proposal.proposal_type.clone());
        batch_id = proposal.batch_id.clone();
        evidence = proposal.evidence.clone();

        let total_votes = proposal.yes_votes.clone() + proposal.no_votes.clone();
        let zero = candid::Nat::from(0u64);

        let yes_percentage = if total_votes > zero {
            let hundred = candid::Nat::from(100u64);
            ((proposal.yes_votes.clone() * hundred) / total_votes).to_string()
        } else {
            "0".to_string() // If no votes have been cast, set the percentage to 0%
        };

        let eth_tx_summary = format!(
            "{}: Proposal {}: {}% yes",
            ic_cdk::id(),
            proposal.id,
            yes_percentage
        );
        println!("Summary for proposal {}: {}", proposal_id, eth_tx_summary);
        Ok(eth_tx_summary)
    })?;

    // if proposal is of type "UpgradeEthFE", upgrade the frontend canister
    if proposal_type.unwrap() == PROPOSAL_TYPE_UPGRADE_ETH_FE {
        // perform upgrade of frontend canister
        // FIXME: For a production-grade implementation this needs to be improved, e.g., using retries to ensure the system remains consistent.
        do_upgrade_eth_fe_canister(batch_id.unwrap(), evidence.unwrap()).await?;
    }

    // Perform the Ethereum transaction and capture the transaction hash
    let transaction_result = eth_transaction(
        TARGET_CONTRACT.into(),
        &ETH_CONTRACT.with(Rc::clone),
        "storeString",
        &[Token::String(eth_tx_summary.clone())],
    ).await?;

    // Update the proposal with the Ethereum transaction hash if the transaction was successful
    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        if let Some(proposal) = proposals.iter_mut().find(|p| p.id == proposal_id) {
            proposal.eth_transaction_hash = Some(transaction_result.clone());
        }
    });

    Ok(transaction_result)
    // Ok("Functions.".to_string())
}

#[update]
async fn get_eth_address() -> String {
    get_self_eth_address().await
}

#[update]
async fn get_my_eth_balance() -> String {
    eth_balance_of(&get_self_eth_address().await, "latest")
        .await
        .to_string()
}

#[init]
fn init(key_id: String) {
    ECDSA_KEY.with(|key| {
        *key.borrow_mut() = key_id;
    });

    // Set up the timer to periodically check and execute proposals
    ic_cdk_timers::set_timer_interval(TIMER_INTERVAL, || {
        ic_cdk::spawn(check_and_execute_proposals());
    });
}

#[post_upgrade]
fn post_upgrade(key_id: String) {
    ECDSA_KEY.with(|key| {
        *key.borrow_mut() = key_id;
    });

    // Re-setup the timer to continue periodic checks after an upgrade
    ic_cdk_timers::set_timer_interval(TIMER_INTERVAL, || {
        ic_cdk::spawn(check_and_execute_proposals());
    });
}

// Function to check and execute proposals if their end time has passed
async fn check_and_execute_proposals() {
    let mut ids_to_execute = Vec::new();

    // Collect proposal IDs synchronously
    PROPOSALS.with(|proposals_ref| {
        let mut proposals = proposals_ref.borrow_mut();
        for proposal in proposals.iter_mut() {
            if proposal.is_open && proposal.proposal_end_timestamp < time() {
                println!("Proposal with ID {} is now closed for voting", proposal.id);
                proposal.is_open = false;
                ids_to_execute.push(proposal.id);
            }
        }
    });

    // Execute each proposal asynchronously
    for id in ids_to_execute {
        println!("Attempting to execute proposal with ID {}", id);
        match execute_proposal(id).await {
            Ok(summary) => println!("Executed proposal {}: {}", id, summary),
            Err(e) => println!("Error executing proposal {}: {}", e, id),
        }
    }
}

fn ecdsa_key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
        name: ECDSA_KEY.with(|key| key.borrow().clone()),
    }
}

#[update]
async fn request_commit_assets(batch_id: u64, evidence: String) {
    println!("Committing frontend assets. Triggered by external call. {}, {}", batch_id, evidence);

    let result = commit_assets(batch_id, evidence).await;
    println!("Result = {:?}", result);
}

#[update]
async fn delete() {
    println!("Deleting frontend asset.");
    
    let result = ic_cdk::call::<_, ()>(
        Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(),
        "delete_asset",
        (DeleteAssetArguments {
            key: Key::from_str("/assets/index-B3YNcDVy.js").unwrap(),
        },),
    )
    .await
    .expect("Failed to call delete_asset");

    println!("Result = {:?}", result);
}

#[update]
async fn commit_assets(batch_id: u64, evidence: String) {
    
    let id = batch_id;
    
    let evidence_bytes = decode(evidence).expect("Decoding of evidence string failed. Be sure you have not copied the trailing \".\" from the command line.");
    
    let result = ic_cdk::call::<_, ()>(
        Principal::from_text("b77ix-eeaaa-aaaaa-qaada-cai").unwrap(),
        //Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(),
        "commit_proposed_batch",
        (CommitProposedBatchArguments {
            batch_id: candid::Nat::from(id),
            evidence: ByteBuf::from(evidence_bytes),
        },),
    )
    .await
    .expect("Failed to call commit");
    
    println!("Result = {:?}", result);
    
}

async fn do_upgrade_eth_fe_canister(batch_id: u64, evidence: Vec<u8>) -> Result<String, String> {
    let fe_canister_principal = Principal::from_text("by6od-j4aaa-aaaaa-qaadq-cai").unwrap();

    let result = ic_cdk::call::<_, ()>(
        fe_canister_principal,
        "commit_proposed_batch",
        (CommitProposedBatchArguments {
            batch_id: candid::Nat::from(batch_id),
            evidence: ByteBuf::from(evidence.clone()),
        },),
    )
    .await;
    //.expect("Failed to call commit on canister {} with batch_id {:?} and evidence {:?}.");
 
    println!("Result = {:?}", result);
    
    match result {
        Ok(r) => { println!("Commit call successful. Canister response = {:?}", r); Ok("Commit call successful.".to_string()) },
        Err(e) => { println!("Error in commit call: {:?}. Failed to execute FE upgrade proposal with batch_id = \"{:?}\" and evidence = \"{:?}\".", e, batch_id, evidence); Err("Error in commit call.".to_string()) },
    }

}

export_candid!();
