import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import swap from "@/api/endpoints/swap";

export default function Swap() {
  const [fromToken, setFromToken] = useState("hyllar");
  const [toToken, setToToken] = useState("hyllar");
  const [fromTokenAmount, setFromTokenAmount] = useState(0);
  const { handleSubmit } = useFormSubmission(swap, {});

  return (
    <form onSubmit={handleSubmit}>
      <Input
        type="text"
        labelText="Username:"
        name="username"
        suffixText=".hydentity"
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

      <Button type="submit">{`Swap ${fromTokenAmount} from ${fromToken} to ${toToken}`}</Button>
    </form>
  );
}
