import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";

export default function Swap() {
  const [fromToken, setFromToken] = useState("hyllar");
  const [toToken, setToToken] = useState("hyllar");
  const [fromTokenAmount, setFromTokenAmount] = useState(0);

  return (
    <form>
      <Input type="text" labelText="Username:" suffixText=".hydentity" />
      <Input type="password" labelText="Password" />

      <TokenSelector token={fromToken} onTokenChange={setFromToken} />
      <TokenSelector token={toToken} onTokenChange={setToToken} />

      <Input
        type="number"
        labelText="Amount"
        placeholder="0"
        onChange={(e) => setFromTokenAmount(Number(e.target.value))}
      />

      <Button type="submit">{`Swap ${fromTokenAmount} from ${fromToken} to ${toToken}`}</Button>
    </form>
  );
}
