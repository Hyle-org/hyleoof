import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import swap from "@/api/endpoints/swap";
import { useHyllar } from "@/hooks/useHyllar";

export default function Swap() {
  const [username, setUsername] = useState("");
  const [fromToken, setFromToken] = useState("hyllar");
  const [toToken, setToToken] = useState("hyllar");
  const [fromTokenAmount, setFromTokenAmount] = useState(0);
  const [message, setMessage] = useState("");
  const { getHydentityBalance } = useHyllar({ contractName: fromToken });

  const { handleSubmit } = useFormSubmission(swap, {
    onMutate: () => {
      setMessage("Swaping...");
    },
    onError: (error) => {
      setMessage(`Failed to swap: ${error.message}`);
    },
    onSuccess: () => {
      setMessage(
        `Swap successful for user ${username}.hydentity`
      );
    },
  });

  return (
    <form onSubmit={handleSubmit}>
      <Input
        type="text"
        labelText="Username:"
        value={username}
        name="username"
        suffixText=".hydentity"
        onChange={(e) => setUsername(e.target.value)}
      />
      <Input type="password" labelText="Password" name="password" />

      <TokenSelector
        token={fromToken}
        name="fromToken"
        onTokenChange={setFromToken}
      />
      <TokenSelector
        token={toToken}
        name="toToken"
        onTokenChange={setToToken}
      />

      <Input
        type="number"
        labelText="Amount"
        name="amount"
        placeholder="0"
        onChange={(e) => setFromTokenAmount(Number(e.target.value))}
      />

<p>{`Balance: ${getHydentityBalance(username) || `Account ${username}.hydentity not found`}`}</p>
      <Button type="submit">{`Swap ${fromTokenAmount} from ${fromToken} to ${toToken}`}</Button>
      <p>{message}</p>
    </form>
  );
}
