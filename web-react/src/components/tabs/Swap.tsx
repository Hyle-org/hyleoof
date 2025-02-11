import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import swap from "@/api/endpoints/swap";
import { useHyllar } from "@/hooks/useHyllar";
import { useMetaMask } from "@/hooks";

export default function Swap() {
  const { account } = useMetaMask();
  const [fromToken, setFromToken] = useState("hyllar");
  const [toToken, setToToken] = useState("hyllar2");
  const [fromTokenAmount, setFromTokenAmount] = useState(0);
  const [message, setMessage] = useState("");
  const { getBalance } = useHyllar({ contractName: fromToken });

  const { handleSubmit } = useFormSubmission(swap, {
    onMutate: () => {
      setMessage("Swaping...");
    },
    onError: (error) => {
      setMessage(`Failed to swap: ${error.message}`);
    },
    onSuccess: () => {
      setMessage(
        `Swap successful`
      );
    },
  });

  return (
    <form onSubmit={handleSubmit}>
      <Input
        type="text"
        labelText="Account:"
        value={account}
        name="account"
        suffixText=""
        readOnly
      />

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

      <p>{`Balance: ${getBalance(account) || `0`}`}</p>
      <Button type="submit">{`Swap ${fromTokenAmount} from ${fromToken} to ${toToken}`}</Button>
      <p>{message}</p>
    </form>
  );
}
