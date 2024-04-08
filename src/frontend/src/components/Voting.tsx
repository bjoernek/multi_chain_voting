import React, { useState, useEffect } from 'react';
import { useActor } from "../ic/Actors";
import Button from "./ui/Button";
import { Proposal } from '../../../declarations/backend/backend.did';
import AddressPill from "./AddressPill";
import PrincipalPill from "./PrincipalPill";
import Spinner from './Spinner';

// Note: This only applies to ETH. For ERC20 tokens the number of decimals is configurable. 
function abbreviateNumber(value: number): string {
  // Convert wei to ETH by dividing by 10^18
  const ethValue = value / 1e18;

  // Define suffixes
  const suffixes = ["ETH", "K ETH", "M ETH", "B ETH", "T ETH"];
  // Calculate the suffix index
  const scale = Math.log10(ethValue) / 3;
  let suffixIndex = Math.floor(scale);
  if (suffixIndex < 0) {
    // Handle cases where ethValue is less than 1 
    return ethValue.toFixed(2) + " ETH";
  }
  
  // Calculate the short value to display
  const shortValue = (ethValue / Math.pow(1000, suffixIndex)).toFixed(2);
  // Remove any trailing .0
  return shortValue.replace(/\.0$/, '') + suffixes[Math.min(suffixIndex, suffixes.length - 1)];
}





export default function Voting() {
  const { actor } = useActor();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [type, setType] = useState('Motion');
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [votingProposals, setVotingProposals] = useState<bigint[]>([]);

  useEffect(() => {
    if (actor) {
      fetchProposals();
    }
  }, [actor]); 



  const handleProposalSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!actor) {
      console.error("Actor is not initialized.");
      return;
    }
    setIsSubmitting(true); // Start the spinner
    try {
      const proposalId = await actor.submit_proposal(title, description, type);
      console.log(`Proposal submitted successfully with ID: ${proposalId}`);

      fetchProposals();
    } catch (error) {
      console.error("Failed to submit proposal:", error);
    }
    setIsSubmitting(false); // Stop the spinner
    setTitle('');
    setDescription('');
    setType('Motion');
  };

  const fetchProposals = async () => {
    if (!actor) {
      console.log("Actor is not initialized.");
      return;
    }

    try {
      const fetchedProposals: Proposal[] = await actor.get_proposals();
      setProposals(fetchedProposals);
    } catch (error) {
      console.error("Failed to fetch proposals:", error);
    }
  };

  const submitVote = async (proposalId: bigint, vote: boolean) => {
    console.log(`Attempting to vote on proposal ${proposalId} with vote: ${vote}`);
    setVotingProposals(current => [...current, proposalId]);


    if (!actor) {
      console.error("Actor is not initialized.");
      setVotingProposals(current => current.filter(id => id !== proposalId));
      return;
    }
    try {
      await actor.vote_on_proposal(proposalId, vote);
      console.log(`Successfully voted on proposal ${proposalId} with vote: ${vote}`);
      fetchProposals();
    } catch (error) {
      console.error(`Failed to submit vote on proposal ${proposalId}:`, error);
    }
    setVotingProposals(current => current.filter(id => id !== proposalId));
  };


  return (

    <div className="w-full max-w-4xl space-y-12 relative">
      {/* Proposal Submission Tile */}
      <div className="w-full max-w-4xl mx-auto border border-gray-600 bg-zinc-900 px-8 py-8 drop-shadow-xl rounded-3xl flex flex-col items-center space-y-6">
        <div className="text-center text-3xl font-bold text-white">
          Submit a Proposal
        </div>

        {/* Overlay Spinner */}
        {isSubmitting && (
          <div className="absolute top-0 left-0 w-full h-full flex justify-center items-center bg-black bg-opacity-50 rounded-3xl">
            <Spinner />
          </div>
        )}

        <form onSubmit={handleProposalSubmit} className={`flex flex-col items-center w-full space-y-4 ${isSubmitting ? 'opacity-50 pointer-events-none' : ''}`}>
          <div className="w-full">
            <label className="block mb-2 text-lg text-gray-400">Title:</label>
            <input value={title} onChange={(e) => setTitle(e.target.value)} className="w-full p-3 rounded-lg border border-gray-600 bg-zinc-700 text-white" />
          </div>
          <div className="w-full">
            <label className="block mb-2 text-lg text-gray-400">Description:</label>
            <textarea value={description} onChange={(e) => setDescription(e.target.value)} className="w-full p-3 h-32 rounded-lg border border-gray-600 bg-zinc-700 text-white" />
          </div>
          <div className="w-full">
            <label className="block mb-2 text-lg text-gray-400">Type:</label>
            <select value={type} onChange={(e) => setType(e.target.value)} className="w-full p-3 rounded-lg border border-gray-600 bg-zinc-700 text-white">
              <option value="Motion">Motion</option>
              <option value="TokenTransfer">Token Transfer</option>
            </select>
          </div>
          <Button className="mt-4 bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded-lg" disabled={isSubmitting}>
            Submit Proposal
          </Button>
        </form>
      </div>


      {/* List of current proposals*/}
      <div className="w-full max-w-7xl mx-auto border border-gray-600 bg-zinc-900 px-5 py-5 drop-shadow-2xl rounded-3xl flex flex-col items-center space-y-8">
        <div className="text-center text-3xl font-bold text-white">Vote on Current Proposals</div>

        {proposals.length > 0 ? (
          <div className="w-full grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-8">
            {proposals
              .slice() // Create a shallow copy to avoid mutating the original array
              .sort((a, b) => Number(b.timestamp - a.timestamp)) // sort proposals, showing most recent ones first
              .map((proposal, index) => (
                <div key={index} className="border border-gray-600 rounded-lg p-6 bg-zinc-800 text-gray-400 hover:bg-zinc-700 transition duration-300 ease-in-out space-y-4 relative">
                  {votingProposals.includes(proposal.id) && (
                    // This spinner is absolutely positioned within the relative container above
                    <div className="absolute inset-0 bg-black bg-opacity-50 flex justify-center items-center z-10">
                      <Spinner />
                    </div>
                  )}
                  <h3 className="text-xl font-semibold text-white">{proposal.title}</h3>
                  <div className="text-gray-300">
                    <span className="font-semibold">Type:</span> <span className="ml-2">{proposal.proposal_type}</span>
                  </div>
                  <div className="text-sm">
                    <span className="font-semibold text-gray-300">ID:</span> {proposal.id.toString()}
                  </div>
                  <p className="text-gray-300"><span className="font-semibold">Description:</span> {proposal.description}</p>

                  {/* Submitter ICP principal and ETH address */}
                  <div className="text-gray-300 font-semibold">Submitter Details:</div>
                  <PrincipalPill principal={proposal.submitter} className="bg-zinc-700" />
                  <AddressPill address={proposal.submitter_eth_address} className="bg-zinc-700" />

                  <p><span className="font-semibold text-gray-300">Timestamp:</span> {new Date(Number(proposal.timestamp) / 1_000_000).toLocaleString()}</p>
                  <p><span className="font-semibold text-gray-300">Blockheight:</span> {proposal.block_height.toString()}</p>
                  <div className="flex justify-between items-center text-sm text-gray-300">
                    <div className="flex gap-4">
                      <button
                        onClick={() => submitVote(proposal.id, true)}
                        className="bg-green-400 hover:bg-green-500 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        disabled={votingProposals.includes(proposal.id)}
                      >
                        Vote Yes
                      </button>
                      <button
                        onClick={() => submitVote(proposal.id, false)}
                        className="bg-red-400 hover:bg-red-500 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        disabled={votingProposals.includes(proposal.id)}
                      >
                        Vote No
                      </button>
                    </div>
                    <div className="ml-6 flex flex-col">
                      <div className="flex items-center mb-2">
                        Yes:
                        <span
                          className="ml-1 font-semibold"
                          title={proposal.yes_votes.toString()}> 
                          {abbreviateNumber(Number(proposal.yes_votes))}
                        </span>
                      </div>
                      <div className="flex items-center">
                        No:
                        <span
                          className="ml-1 font-semibold"
                          title={proposal.no_votes.toString()}> 
                          {abbreviateNumber(Number(proposal.no_votes))}
                        </span>
                      </div>
                    </div>


                  </div>
                </div>
              ))}

          </div>
        ) : (
          <p className="text-gray-400">No proposals found.</p>
        )}
        <Button onClick={fetchProposals} className="mt-4 bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded-lg">
          Refresh Proposals
        </Button>
      </div>

    </div >
  );
}
