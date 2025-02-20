import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import TokenSelector from "@/components/TokenSelector";
import faucet from "@/api/endpoints/faucet";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useHyllar } from "@/hooks/useHyllar";
import { useMetaMask } from "@/hooks";
import { useFetchEvents } from "@/hooks/useFetchEvents";

const FAUCET_AMOUNT = 10;

export default function Faucet() {
  const { account } = useMetaMask();
  const [recipient, setRecipient] = useState(account);
  const [token, setToken] = useState("hyllar");
  const [message, setMessage] = useState("");
  const { getBalance, updateHyllarState } = useHyllar({ contractName: token });
  const fetchEvents = useFetchEvents((e: string) => setMessage(e), () => {
    updateHyllarState();
    setTimeout(() => setMessage("✅ Enjoy you tokens!"), 2000);
  });

  const { handleSubmit } = useFormSubmission(faucet, {
    onMutate: () => {
      setMessage("Fauceting...");
    },
    onError: (error) => {
      setMessage(`Failed to faucet: ${error.message}`);
    },
    onSuccess: async (tx) => {
      setMessage(`Faucet successful, token ${token}`);
      fetchEvents(tx as string);
    },
  });


  return (
    <>
      <form onSubmit={handleSubmit}>
        <TokenSelector token={token} onTokenChange={setToken} />
        <Input
          type="text"
          labelText=""
          suffixText=""
          value={recipient}
          name="account"
          onChange={(e) => setRecipient(e.target.value)}
        />

        <Button type="submit">{`Faucet ${FAUCET_AMOUNT} hyllar`}</Button>
      </form>

      <div className="state">
        <p>{`Balance: ${getBalance(account) || `0`}`}</p>
        <p>{message}</p>
      </div>
    </>
  );
}
