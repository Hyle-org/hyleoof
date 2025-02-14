import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import TokenSelector from "@/components/TokenSelector";
import faucet from "@/api/endpoints/faucet";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useHyllar } from "@/hooks/useHyllar";
import { useMetaMask } from "@/hooks";

const FAUCET_AMOUNT = 10;

export default function Faucet() {
  const { account } = useMetaMask();
  const [recipient, setRecipient] = useState(account);
  const [token, setToken] = useState("hyllar");
  const [message, setMessage] = useState("");
  const { getBalance: getBalance } = useHyllar({ contractName: token });

  const { handleSubmit } = useFormSubmission(faucet, {
    onMutate: () => {
      setMessage("Fauceting...");
    },
    onError: (error) => {
      setMessage(`Failed to faucet: ${error.message}`);
    },
    onSuccess: () => {
      setMessage(`Faucet successful, token ${token}`);
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
        <p>{message}</p>
        <p>{`Balance: ${getBalance(account) || `0`}`}</p>
      </div>
    </>
  );
}
