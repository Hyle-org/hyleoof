import { useEffect, useState } from "react";
import Register from "@/components/tabs/Register";
import Faucet from "@/components/tabs/Faucet";
import Transfer from "@/components/tabs/Transfer";
import Swap from "@/components/tabs/Swap";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useMetaMask } from "./hooks/useMetaMask";
import { useRequestSnap } from "./hooks/useRequestSnap";
import { useInvokeSnap } from "./hooks/useInvokeSnap";
import { isLocalSnap } from "./utils/snap";
import { defaultSnapOrigin, idContractName } from "./config";
import { ConnectButton, InstallFlaskButton } from "./components/Buttons";
import Copiable from "./components/Copiable";
import SnapToggle from "./components/ui/SnapToggle";

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
  const { isFlask, providerDetected, provider, installedSnap, useSnap, setUseSnap, account, setAccount } = useMetaMask();
  const requestSnap = useRequestSnap();
  const invokeSnap = useInvokeSnap();
  const [autoconnect, setAutoconnect] = useState(true);

  const isMetaMaskReady = useSnap ? (isLocalSnap(defaultSnapOrigin)
    ? isFlask
    : providerDetected) : providerDetected;

  const handleConnect = async () => {
    console.log("connecting")
    await requestSnap();
    console.log("get account")
    await getAccount();
  };

  const getAccount = async () => {
    if (useSnap) {
      const { account } = await invokeSnap({ method: "get_account" }) as { account: string, nonce: number };
      setAccount(account);
    } else {
      const accounts = await provider?.request({ method: "eth_requestAccounts" }) as string[];
      console.log("accounts", accounts)
      const account = `${accounts[0]}.${idContractName}`;
      setAccount(account);
    }
    setAutoconnect(false);
  };

  const handleToggle = () => {
    setUseSnap((prev) => !prev);
  };

  if ((!useSnap || installedSnap) && autoconnect) { getAccount(); }

  const [activeTab, setActiveTab] = useState<TabOption>(TabOption.Faucet);
  const ActiveComponent = TabComponents[activeTab];

  return (
    <QueryClientProvider client={queryClient}>
      <div className="container">
        <div className="header">
          <SnapToggle isToggled={useSnap} onToggle={handleToggle} />
          {useSnap && !isMetaMaskReady && (
            <InstallFlaskButton />
          )}
          {useSnap && !installedSnap && (
            <ConnectButton
              onClick={handleConnect}
              useSnap={useSnap}
              disabled={!isMetaMaskReady}
            />
          )}
          {account && (
            <div>
              <p>Connected with:</p>
              <Copiable text={account} size={40} />
            </div>
          )}
          {(!useSnap || installedSnap) && !account && (
            <ConnectButton useSnap={useSnap} onClick={getAccount} />
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
