import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import TokenSelector from "@/components/TokenSelector";

export default function Faucet() {
  const [hydentity, setHydentity] = useState("");
  return (
    <>
      <form>
        <TokenSelector />
        <Input
          type="text"
          labelText="Username"
          suffixText=".hydentity"
          value={hydentity}
          onChange={(e) => setHydentity(e.target.value)}
        />

        <Button type="submit">{`Faucet 10 hyllar to ${hydentity}.hydentity`}</Button>
      </form>
      <div className="state">
        <p>Total supply: 100000000000</p>
        <p>Balance: Account .hydentity not found</p>
      </div>
    </>
  );
}
