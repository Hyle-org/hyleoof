import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import TokenSelector from "@/components/TokenSelector";
import faucet from "@/api/endpoints/faucet";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useHyllar } from "@/hooks/useHyllar";

const FAUCET_AMOUNT = 10;

export default function Faucet() {
  const [username, setUsername] = useState("");
  const [token, setToken] = useState("hyllar");
  const [message, setMessage] = useState("");
  const { getTotalSupply, getHydentityBalance } = useHyllar({ contractName: token });

  const { handleSubmit } = useFormSubmission(faucet, {
    onMutate: () => {
      setMessage("Fauceting...");
    },
    onError: (error) => {
      setMessage(`Failed to faucet: ${error.message}`);
    },
    onSuccess: () => {
      setMessage(`Faucet successful for user ${username}.hydentity, token ${token}`);
    },
  });

  return (
    <>
      <form onSubmit={handleSubmit}>
        <TokenSelector token={token} onTokenChange={setToken} />
        <Input
          type="text"
          labelText="Username"
          suffixText=".hydentity"
          value={username}
          name="username"
          onChange={(e) => setUsername(e.target.value)}
        />

        <Button type="submit">{`Faucet ${FAUCET_AMOUNT} hyllar to ${username}.hydentity`}</Button>
      </form>

      <div className="state">
        <p>{message}</p>
        <p>{`Token supply: ${getTotalSupply() || "Loading..."}`}</p>
        <p>{`Balance: ${getHydentityBalance(username) || "Account .hydentity not found"}`}</p>
      </div>
    </>
  );
}
