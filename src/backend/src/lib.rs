mod service;
mod user_profile;

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use user_profile::UserProfile;

use ic_cdk::api::{caller, time};
use ic_cdk_macros::*;
use ic_cdk::println;
use std::collections::HashMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type GetAddressResponse = Result<String, String>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static USER_PROFILES: RefCell<StableBTreeMap<String, UserProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static SIWE_PROVIDER_CANISTER: RefCell<Option<Principal>>  = RefCell::new(None);

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
    yes_votes: u64,
    no_votes: u64,
}

#[update]
fn submit_proposal(title: String, description: String, proposal_type: String, eth_address: String) -> u64 {
    let submitter = caller().to_text();
    let submitter_eth_address = eth_address;
    let timestamp = time();

    PROPOSALS.with(|proposals| {
        let mut proposals = proposals.borrow_mut();
        let new_id = proposals.len() as u64 + 1; // Simple ID generation
        let proposal = Proposal {
            id: new_id,
            title,
            description,
            proposal_type,
            submitter,
            submitter_eth_address,
            timestamp,
            yes_votes: 0, // No votes yet
            no_votes: 0,  // No votes yet
        };

        proposals.push(proposal);
        new_id // Returning the ID of the new proposal
    })
}

#[query]
fn get_proposals() -> Vec<Proposal> {
    PROPOSALS.with(|proposals_ref| {
        proposals_ref.borrow().clone()
    })
}


#[update]
fn vote_on_proposal(proposal_id: u64, vote: bool) -> Result<(), String> {
    let voter_principal = caller().to_text();
    println!("Received vote: {}, from principal: {}, for proposal: {}", vote, voter_principal, proposal_id);

    // First, check if the proposal exists.
    let exists = PROPOSALS.with(|proposals| {
        proposals.borrow().iter().any(|p| p.id == proposal_id)
    });

    if !exists {
        println!("Proposal not found for ID: {}", proposal_id);
        return Err("Proposal not found".to_string());
    }

    // Then, record the vote, ensuring each principal votes only once per proposal.
    let already_voted = VOTES.with(|votes| {
        let mut votes = votes.borrow_mut();
        let proposal_votes = votes.entry(proposal_id).or_insert_with(HashMap::new);

        if proposal_votes.contains_key(&voter_principal) {
            println!("Principal: {} has already voted on proposal: {}", voter_principal, proposal_id);
            true
        } else {
            // Record the new vote.
            proposal_votes.insert(voter_principal.clone(), vote);
            println!("Vote: {} recorded for principal: {} on proposal: {}", vote, voter_principal, proposal_id);
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
                proposal.yes_votes += 1;
                println!("Incremented yes votes for proposal ID: {}", proposal_id);
            } else {
                proposal.no_votes += 1;
                println!("Incremented no votes for proposal ID: {}", proposal_id);
            }
        }
    });

    Ok(())
}

