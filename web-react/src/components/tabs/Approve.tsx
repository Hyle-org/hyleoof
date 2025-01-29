import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import { useState } from "react";
import Button from "../ui/Button";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import approve from "@/api/endpoints/approve";
import { useHyllar } from "@/hooks/useHyllar";

export default function Approve() {
  const [username, setUsername] = useState("");
  const [spender, setSpender] = useState("");
  const [amount, setAmount] = useState(0);
  const [token, setToken] = useState("hyllar");
  const [message, setMessage] = useState("");
  const { getHydentityBalance } = useHyllar({ contractName: token });

  const { handleSubmit } = useFormSubmission(approve, {
    onMutate: () => {
      setMessage("Approving...");
    },
    onError: (error) => {
      setMessage(`Failed to approve: ${error.message}`);
    },
    onSuccess: () => {
      setMessage(
        `Approve successful for user ${username}.hydentity`
      );
    },
  });

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
        value="amm"
        disabled={true}
        onChange={(e) => setSpender(e.target.value)}
      />
      <Input
        type="number"
        labelText="Amount"
        name="amount"
        placeholder="0"
        onChange={(e) => setAmount(Number(e.target.value))}
      />

      <p>{`Balance: ${getHydentityBalance(username) || `Account ${username}.hydentity not found`}`}</p>
      <Button type="submit">
        {`Approve ${amount} ${token} from ${username}.hydentity to ${spender}.hydentity`}
      </Button>
      <p>{message}</p>
    </form>
  );
}
