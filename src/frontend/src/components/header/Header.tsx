import EthButton from "./EthButton";
import IdentityButton from "./IdentityButton";
import SessionButton from "./SessionButton";

export default function Header() {
  return (
    <div className="flex flex-col justify-between w-full gap-10 p-5 md:flex-row">
      <div className="hidden text-3xl font-bold text-center md:block">
        Multi-chain Governance: On-Chain Voting with Ethereum and Internet Computer
      </div>
      <div className="flex items-center justify-center gap-5 text-sm md:text-base">
        <IdentityButton />
        <EthButton />
        <SessionButton />
      </div>
      <div className="block text-base font-bold text-center md:hidden">
        Multi-chain Governance: On-Chain Voting with Ethereum and Internet Computer
      </div>
    </div>
  );
}
