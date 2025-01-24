import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import TokenSelector from "@/components/TokenSelector";
import faucet from "@/api/endpoints/faucet";
import { useFormSubmission } from "@/hooks/useFormSubmission";

const FAUCET_AMOUNT = 10;

export default function Faucet() {
  const [username, setUsername] = useState("");
  const [token, setToken] = useState("hyllar");
  const [tokenSupply, setTokenSupply] = useState(100000000000);
  // const [userBalance, setUserBalance] = useState();
  const { handleSubmit } = useFormSubmission(faucet, {});

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
        <p>{`Token supply: ${tokenSupply}`}</p>
        <p>Balance: Account .hydentity not found</p>
      </div>
    </>
  );
}
