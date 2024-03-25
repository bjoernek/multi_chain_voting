import { useState } from 'react';
import { backend } from '../../../declarations/backend';
import { useActor } from "../ic/Actors";
import { useAccount } from "wagmi";
import Button from "./ui/Button";

export default function Voting() {
  //const [actor, setActor] = useState(backend);
  const { actor } = useActor();
  const { address, isConnected, isConnecting } = useAccount();

  // State for proposal form
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [type, setType] = useState('Motion'); // Default to 'Motion'

  const [proposals, setProposals] = useState([]);

  // Handle submission of the new proposal
  const handleProposalSubmit = async (e) => {
    e.preventDefault();
    if (!actor) {
      console.error("Actor is not initialized.");
      return;
    }
    try {
      // Call the submit_proposal method on the actor
      const proposalId = await actor.submit_proposal(title, description, type );
      console.log(`Proposal submitted successfully with ID: ${proposalId}`);

      //Fetch updated list of proposals after submission
      fetchProposals();
    } catch (error) {
      console.error("Failed to submit proposal:", error);
    }

    // Reset the proposal form
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
      const fetchedProposals = await actor.get_proposals();
      setProposals(fetchedProposals);
    } catch (error) {
      console.error("Failed to fetch proposals:", error);
    }
  };

  const submitVote = async (proposalId, vote) => {
    console.log(`Attempting to vote on proposal ${proposalId} with vote: ${vote}`);
    try {
      await actor.vote_on_proposal(proposalId, vote);
      console.log(`Successfully voted on proposal ${proposalId} with vote: ${vote}`);
      fetchProposals(); // Refresh proposals to update vote tallies
    } catch (error) {
      console.error(`Failed to submit vote on proposal ${proposalId}:`, error);
    }
  };


  return (
    <div> 
        {/* New Proposal Submission Tile */}
        <div className="w-full max-w-2xl border-zinc-700/50 border-[1px] bg-zinc-900 px-5 py-5 drop-shadow-xl rounded-3xl flex flex-col items-center">
          <div className="text-center text-2xl font-bold py-8 md:px-8">
            Submit a proposal
          </div>

          <form onSubmit={handleProposalSubmit} className="flex flex-col items-center w-full">
            <label>Title:</label>
            <input value={title} onChange={(e) => setTitle(e.target.value)} className="mt-3 w-full" />
            <label>Description:</label>
            <textarea value={description} onChange={(e) => setDescription(e.target.value)} className="mt-3 w-full" />
            <label>Type:</label>
            <select value={type} onChange={(e) => setType(e.target.value)} className="mt-3 w-full" >
              <option value="Motion">Motion</option>
              <option value="TokenTransfer">Token Transfer</option>
            </select>
            <Button >
              {"Submit Proposal"}
            </Button>
          </form>
        </div>
      

      <div className="w-full max-w-2xl border-zinc-700/50 border-[1px] bg-zinc-900 px-5 py-5 drop-shadow-xl rounded-3xl flex flex-col items-center">
        <div className="flex flex-col items-center w-full gap-10 py-8 md:px-8">
          <div className="text-2xl font-bold">Current Proposals</div>
          <div className="flex flex-col items-center gap-5">
            {proposals.length > 0 ? (
              proposals.map((proposal, index) => (
                <div key={index} className="border border-gray-300 rounded-md p-4 mb-2.5 bg-gray-100 text-gray-700">
                  <h3>Title: {proposal.title}</h3>
                  <p>Id: {proposal.id.toString()}</p>
                  <p>Description: {proposal.description}</p>
                  <p>Type: {proposal.proposal_type}</p>
                  <p>Submitter ICP principal: {proposal.submitter}</p>
                  <p>Submitter ETH address: {proposal.submitter_eth_address}</p>
                  <p>Timestamp: {new Date(Number(proposal.timestamp) / 1_000_000).toLocaleString()}</p>
                  {/* Display vote tally */}
                  <p>Yes Votes: {proposal.yes_votes.toString()}</p>
                  <p>No Votes: {proposal.no_votes.toString()}</p>
                  {/* Buttons for voting */}
                  <div className="flex items-center justify-center gap-5 text-sm md:text-base text-zinc-300">
                    <Button onClick={() => submitVote(proposal.id, true)} variant="dark">
                      {"Vote Yes"}
                    </Button>
                    <Button onClick={() => submitVote(proposal.id, false)} variant="dark">
                      {"Vote No"}
                    </Button>
                  </div>
                </div>
              ))
            ) : (
              <p>No proposals found.</p>
            )}
          </div>
          <Button onClick={fetchProposals} >
            {"Refresh Proposals"}
          </Button>
        </div>
      </div>
    </div>
  );
}
