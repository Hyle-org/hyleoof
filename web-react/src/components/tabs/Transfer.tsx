import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import Button from "@/components/ui/Button";
import { useState } from "react";
import transfer from "@/api/endpoints/transfer";
import { useFormSubmission } from "@/hooks/useFormSubmission";

export default function Transfer() {
  const [username, setUsername] = useState("");
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState(0);
  const [token, setToken] = useState("hyllar");
  const { handleSubmit } = useFormSubmission(transfer, {});

  return (
    <form onSubmit={handleSubmit}>
      <TokenSelector token={token} onTokenChange={setToken} />

      <Input
        type="text"
        labelText="Username"
        name="username"
        suffixText=".hydentity"
        onChange={(e) => setUsername(e.target.value)}
      />
      <Input name="password" type="password" labelText="Password" />
      <Input
        type="text"
        name="recipient"
        labelText="Recipient"
        suffixText=".hydentity"
        onChange={(e) => setRecipient(e.target.value)}
      />
      <Input
        type="number"
        name="amount"
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
