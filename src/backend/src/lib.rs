mod declarations;
mod eth_rpc;
mod service;
mod user_profile;

use crate::eth_rpc::balance_of;
use candid::{CandidType, Deserialize, Nat};
use eth_rpc::{eth_transaction, get_self_eth_address, latest_block_number};
use ethers_core::abi::{Contract, Token};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use std::rc::Rc;
use user_profile::UserProfile;

use ic_cdk::api::{caller, time};
use ic_cdk::{println, query, update};
use std::collections::HashMap;

pub const TARGET_CONTRACT: &str = "0xcd76a64b5914aca2b59615a66af9073bb25b5008";
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
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct Proposal {
    id: u64,
    title: String,
    description: String,
    proposal_type: String,
    submitter: String,
    submitter_eth_address: String,
    timestamp: u64,
    accepting_votes: bool,
    yes_votes: Nat,
    no_votes: Nat,
    block_height: String,
}

#[update]
async fn submit_proposal(title: String, description: String, proposal_type: String) -> u64 {
    let submitter = caller().to_text();
    // Initialize submitter_eth_address as an empty string or an appropriate default value
    let mut submitter_eth_address: String = "".to_string();
    let timestamp = time();

    // Attempt to get the address asynchronously
    match service::save_my_profile::get_address().await {
        Ok(address) => {
            println!("Address: {}", address);
            // Store the address in submitter_eth_address if successful
            submitter_eth_address = address;
        }
        Err(e) => {
            println!("Error retrieving address: {}", e);
            // Here you may choose to handle the error, like defaulting to a fallback address, or stopping execution
            // For this example, we'll just log the error. You might want to return or handle differently in real code.
        }
    }

    let (_, block_height) = latest_block_number().await;
    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        let new_id = proposals.len() as u64 + 1; // Simple ID generation
        let proposal = Proposal {
            id: new_id,
            title,
            description,
            proposal_type,
            submitter,
            submitter_eth_address, // This will be empty or contain the address from get_address()
            timestamp,
            accepting_votes: true,
            yes_votes: 0_usize.into(), // No votes yet
            no_votes: 0_usize.into(),  // No votes yet
            block_height,
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

    // Attempt to find the proposal
    let block_number = PROPOSALS.with(|proposals| {
        proposals.borrow().iter().find(|&p| p.id == proposal_id).map_or_else(
            || Err("Proposal not found".to_string()),
            |proposal| Ok(proposal.block_height.clone()),
        )
    })?;
    
    let voter = match service::save_my_profile::get_address().await {
        Ok(address) => address,
        Err(e) => {
            ic_cdk::trap(&format!("Error retrieving address: {}", e));
        }
    };

    let voting_power = balance_of(&voter, &block_number).await;

    // Then, record the vote, ensuring each principal votes only once per proposal.
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

    // Optionally, update the proposal's vote tally immediately.
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
    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        let Some(proposal) = proposals.iter_mut().find(|p| p.id == proposal_id) else {
            return Err(format!("Proposal {proposal_id} not found."));
        };
        if !proposal.accepting_votes {
            return Err(format!("Proposal {proposal_id} already executed"));
        }
        proposal.accepting_votes = false;
        Ok(())
    })?;

    Ok(eth_transaction(
        TARGET_CONTRACT.into(),
        &ETH_CONTRACT.with(Rc::clone),
        "executeProposal",
        &[Token::Uint(proposal_id.into())],
    )
    .await)
}

#[update]
async fn get_eth_address() -> String {
    get_self_eth_address().await
}
