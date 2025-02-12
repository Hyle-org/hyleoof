import { useState } from "react";
import Register from "@/components/tabs/Register";
import Faucet from "@/components/tabs/Faucet";
import Transfer from "@/components/tabs/Transfer";
import Swap from "@/components/tabs/Swap";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useMetaMask } from "./hooks/useMetaMask";
import { useRequestSnap } from "./hooks/useRequestSnap";
import { useInvokeSnap } from "./hooks/useInvokeSnap";
import { isLocalSnap } from "./utils/snap";
import { defaultSnapOrigin } from "./config";
import { useMetaMaskContext } from "./hooks/MetamaskContext";
import { ConnectButton, DisconnectButton, InstallFlaskButton } from "./components/Buttons";
import { shortenString } from "./utils/shortenString";
import { useRequest } from "./hooks";
import { a } from "vitest/dist/chunks/suite.BJU7kdY9.js";

enum TabOption {
  Register = "Register",
  Faucet = "Faucet",
  Transfer = "Transfer",
  Swap = "Swap",
}

const TabComponents: Record<TabOption, React.FC> = {
  [TabOption.Register]: () => <Register />,
  [TabOption.Faucet]: () => <Faucet />,
  [TabOption.Transfer]: () => <Transfer />,
  [TabOption.Swap]: () => <Swap />,
};

const queryClient = new QueryClient();

function App() {
  const { error } = useMetaMaskContext();
  const { isFlask, snapsDetected, installedSnap, account, setAccount } = useMetaMask();
  const requestSnap = useRequestSnap();
  const request = useRequest();
  const [autoconnect, setAutoconnect] = useState(true);

  const isMetaMaskReady = isLocalSnap(defaultSnapOrigin)
    ? isFlask
    : snapsDetected;

  const handleConnect = async () => {
    console.log("connecting")
    await requestSnap();
    console.log("get account")
    await getAccount();
  };

  const getAccount = async () => {
    setAutoconnect(false);
    const ethAccounts = await request({ method: "eth_requestAccounts" }) as string[];
    console.log(ethAccounts);
    setAccount(ethAccounts[0]);
  };

  if (installedSnap && autoconnect) { getAccount(); }

  const [activeTab, setActiveTab] = useState<TabOption>(TabOption.Faucet);
  const ActiveComponent = TabComponents[activeTab];

  return (
    <QueryClientProvider client={queryClient}>
      <div className="container">
        <div className="header">
          {!isMetaMaskReady && (
            <InstallFlaskButton />
          )}
          {!installedSnap && (
            <ConnectButton
              onClick={handleConnect}
              disabled={!isMetaMaskReady}
            />
          )}
          {installedSnap && account && (
            <div>
              <p>Connected with:</p>
              <p>{shortenString(account, 45)}</p>
            </div>
          )}
          {
            installedSnap && !account && (
              <ConnectButton onClick={getAccount} />
            )
          }
        </div>
        <div className="tabs">
          {Object.values(TabOption).map((tab) => (
            <button
              key={tab}
              className={`tab ${activeTab === tab ? "active" : ""}`}
              onClick={() => setActiveTab(tab)}
            >
              {tab}
            </button>
          ))}
        </div>
        <div className="content">
          <ActiveComponent />
        </div>
      </div>
    </QueryClientProvider>
  );
}

export default App;
