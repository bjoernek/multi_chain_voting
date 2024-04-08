import { useAccount, useNetwork } from "wagmi";

import AddressPill from "../AddressPill";
import Button from "../ui/Button";
import ConnectButton from "./ConnectButton";
import LoginButton from "./LoginButton";
import { faWaveSquare } from "@fortawesome/free-solid-svg-icons";
import { isChainIdSupported } from "../../wagmi/is-chain-id-supported";
import { useSiweIdentity } from "ic-use-siwe-identity";
import { useEffect } from "react";
import toast from "react-hot-toast";

export default function LoginPage(): React.ReactElement {
  const { isConnected, address } = useAccount();
  const { chain } = useNetwork();
  const { prepareLogin, isPrepareLoginIdle, prepareLoginError, loginError } =
    useSiweIdentity();

  /**
   * Preload a Siwe message on every address change.
   */
  useEffect(() => {
    if (!isPrepareLoginIdle || !isConnected || !address) return;
    prepareLogin();
  }, [isConnected, address, prepareLogin, isPrepareLoginIdle]);

  /**
   * Show an error toast if the prepareLogin() call fails.
   */
  useEffect(() => {
    if (prepareLoginError) {
      toast.error(prepareLoginError.message, {
        position: "bottom-right",
      });
    }
  }, [prepareLoginError]);

  /**
   * Show an error toast if the login call fails.
   */
  useEffect(() => {
    if (loginError) {
      toast.error(loginError.message, {
        position: "bottom-right",
      });
    }
  }, [loginError]);

  return (
    <div className="flex flex-col items-center justify-center w-full h-screen gap-10">

      <div className="flex items-center justify-center gap-5 md:gap-20">
        <img alt="ic" className="w-20 h-20 md:w-28 md:h-28" src="/ic.svg" />
        <img alt="eth" className="w-20 h-20 md:w-32 md:h-32" src="/eth.svg" />
      </div>

      <div className="px-10 text-xl font-bold text-center md:text-3xl">
        Multi-chain Governance: On-Chain Voting with Ethereum and Internet Computer
      </div>
      <div className="w-80 md:w-96 border-zinc-700/50 border-[1px] bg-zinc-900 drop-shadow-xl rounded-3xl flex flex-col items-center py-5 mt-8 px-5 mx-10">
        <div className="flex flex-col items-center w-full gap-10 p-8">
          <div className="flex items-center justify-center w-full gap-5">
            <div className="items-center justify-center hidden w-8 h-8 text-xl font-bold rounded-full md:flex bg-zinc-300 text-zinc-800">
              1
            </div>
            <div>
              {!isConnected && <ConnectButton />}
              {isConnected && isChainIdSupported(chain?.id) && (
                <AddressPill
                  address={address}
                  className="justify-center w-44"
                />
              )}
              {isConnected && !isChainIdSupported(chain?.id) && (
                <Button disabled icon={faWaveSquare} variant="outline">
                  Unsupported Network
                </Button>
              )}
            </div>
          </div>
          <div className="flex items-center justify-center w-full gap-5">
            <div className="items-center justify-center hidden w-8 h-8 text-xl font-bold rounded-full md:flex bg-zinc-300 text-zinc-800">
              2
            </div>
            <div>
              {" "}
              <LoginButton />
            </div>
          </div>
        </div>
      </div>
      <div className="text-xl text-gray-400 mt-2">
        Powered by <a href="https://internetcomputer.org/multichain" className="text-blue-500 hover:text-blue-700" target="_blank" rel="noopener noreferrer">Chain Fusion Technology</a>
      </div>
    </div>
  );
}
