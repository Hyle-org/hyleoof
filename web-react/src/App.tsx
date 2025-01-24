import { useState } from "react";
import Register from "@/components/tabs/Register";
import Faucet from "@/components/tabs/Faucet";
import Transfer from "@/components/tabs/Transfer";
import Approve from "@/components/tabs/Approve";
import Swap from "@/components/tabs/Swap";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

enum TabOption {
  Register = "Register",
  Faucet = "Faucet",
  Transfer = "Transfer",
  Approve = "Approve",
  Swap = "Swap",
}

const TabComponents: Record<TabOption, React.FC> = {
  [TabOption.Register]: () => <Register />,
  [TabOption.Faucet]: () => <Faucet />,
  [TabOption.Transfer]: () => <Transfer />,
  [TabOption.Approve]: () => <Approve />,
  [TabOption.Swap]: () => <Swap />,
};

const queryClient = new QueryClient();

function App() {
  const [activeTab, setActiveTab] = useState<TabOption>(TabOption.Approve);
  const ActiveComponent = TabComponents[activeTab];

  return (
    <QueryClientProvider client={queryClient}>
      <div className="container">
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
