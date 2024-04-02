import React, { useState, useEffect } from 'react';
import { useActor } from "../ic/Actors";
import Button from "./ui/Button";
import { Proposal } from '../../../declarations/backend/backend.did';
import AddressPill from "./AddressPill";
import PrincipalPill from "./PrincipalPill";
import Spinner from './Spinner';


export default function Voting() {
  const { actor } = useActor();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [type, setType] = useState('Motion');
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);


  useEffect(() => {
    if (actor) {
      fetchProposals();
    }
  }, [actor]); // This assumes actor is stable and only initialized once; adjust based on your useActor hook



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
    if (!actor) {
      console.error("Actor is not initialized.");
      return;
    }
    try {
      await actor.vote_on_proposal(proposalId, vote);
      console.log(`Successfully voted on proposal ${proposalId} with vote: ${vote}`);
      fetchProposals();
    } catch (error) {
      console.error(`Failed to submit vote on proposal ${proposalId}:`, error);
    }
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
          <div className="w-full grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {proposals
              .slice() // Create a shallow copy to avoid mutating the original array
              .sort((a, b) => Number(b.timestamp - a.timestamp)) // sort proposals, showing most recent ones first
              .map((proposal, index) => (
                <div key={index} className="border border-gray-600 rounded-lg p-6 bg-zinc-800 text-gray-400 hover:bg-zinc-700 transition duration-300 ease-in-out space-y-4">
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
                      <button onClick={() => submitVote(proposal.id, true)} className="bg-green-400 hover:bg-green-500 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline">
                        Vote Yes
                      </button>
                      <button onClick={() => submitVote(proposal.id, false)} className="bg-red-400 hover:bg-red-500 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline">
                        Vote No
                      </button>
                    </div>
                    <div className="flex items-center ml-6"> {/* Added margin-left here */}
                      <div className="mr-2">Yes: <span className="font-semibold">{proposal.yes_votes.toString()}</span></div>
                      <div>No: <span className="font-semibold">{proposal.no_votes.toString()}</span></div>
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
