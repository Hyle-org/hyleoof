import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import approve from "@/api/endpoints/approve";

export default function Approve() {
  const [username, setUsername] = useState("");
  const [spender, setSpender] = useState("");
  const [amount, setAmount] = useState(0);
  const [token, setToken] = useState("hyllar");
  const { handleSubmit } = useFormSubmission(approve, {});

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
      <Input type="password" name="password" labelText="Password" />
      <Input
        type="text"
        labelText="Spender"
        name="spender"
        suffixText=".hydentity"
        onChange={(e) => setSpender(e.target.value)}
      />
      <Input
        type="number"
        labelText="Amount"
        name="amount"
        placeholder="0"
        onChange={(e) => setAmount(Number(e.target.value))}
      />

      <Button type="submit">
        {`Approve ${amount} ${token} from ${username}.hydentity to ${spender}.hydentity`}
      </Button>
    </form>
  );
}
