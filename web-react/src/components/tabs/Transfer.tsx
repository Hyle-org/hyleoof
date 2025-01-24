import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import Button from "@/components/ui/Button";
import { useState } from "react";

export default function Transfer() {
  const [username, setUsername] = useState("");
  const [recipient, setRecipient] = useState("");
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
        labelText="Recipient"
        suffixText=".hydentity"
        onChange={(e) => setRecipient(e.target.value)}
      />
      <Input
        type="number"
        labelText="Amount"
        placeholder="0"
        onChange={(e) => setAmount(Number(e.target.value))}
      />

      <p>Your balance: 0</p>

      <Button type="submit">
        {`Transfer ${amount} ${token} from ${username}.hydentity to ${recipient}.hydentity`}
      </Button>
    </form>
  );
}
