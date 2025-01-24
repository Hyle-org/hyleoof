import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";

export default function Approve() {
  const [username, setUsername] = useState("");
  const [spender, setSpender] = useState("");
  const [amount, setAmount] = useState(0);
  const [token, setToken] = useState("hyllar");

  return (
    <form>
      <TokenSelector token={token} onTokenChange={setToken} />

      <Input
        type="text"
        labelText="Username"
        suffixText=".hydentity"
        onChange={(e) => setUsername(e.target.value)}
      />
      <Input type="password" labelText="Password" />
      <Input
        type="text"
        labelText="Spender"
        suffixText=".hydentity"
        onChange={(e) => setSpender(e.target.value)}
      />
      <Input
        type="number"
        labelText="Amount"
        placeholder="0"
        onChange={(e) => setAmount(Number(e.target.value))}
      />

      <Button type="submit">
        {`Approve ${amount} ${token} from ${username}.hydentity to ${spender}.hydentity`}
      </Button>
    </form>
  );
}
