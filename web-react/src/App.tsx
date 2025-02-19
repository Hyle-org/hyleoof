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
import { ConnectButton, InstallFlaskButton } from "./components/Buttons";
import Copiable from "./components/Copiable";

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
  const { isFlask, snapsDetected, installedSnap, account, setAccount, setNonce } = useMetaMask();
  const requestSnap = useRequestSnap();
  const invokeSnap = useInvokeSnap();
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
    const { account, nonce } = await invokeSnap({ method: "get_account" }) as { account: string, nonce: number };
    setNonce(nonce);
    setAccount(account);
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
              <Copiable text={account} size={40} />
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
