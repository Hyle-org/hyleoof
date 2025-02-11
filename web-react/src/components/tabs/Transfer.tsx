import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import Button from "@/components/ui/Button";
import { FormEvent, useState } from "react";
import transfer from "@/api/endpoints/transfer";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useHyllar } from "@/hooks/useHyllar";
import { useInvokeSnap, useMetaMask } from "@/hooks";
import { signMessage } from "@/utils/sign";
import { idContractName } from "@/config";
import { HYLE_PROVER_URL } from "@/config/contract";

export type TxHash = string;
export type BlockHeight = number;
export type ContractName = string;
export type StateDigest = string;
export type Identity = string;

export interface Proof {
  tx_hash: TxHash;
  contract_name: ContractName;
  identity: Identity;
  signature: string;
}

export default function Transfer() {
  const { account } = useMetaMask();
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState(0);
  const [token, setToken] = useState("hyllar");
  const [message, setMessage] = useState("");
  const { getBalance } = useHyllar({ contractName: token });
  const invokeSnap = useInvokeSnap();

  const { handleSubmit } = useFormSubmission(transfer, {
    onMutate: () => {
      setMessage("Transferring...");
    },
    onError: (error) => {
      setMessage(`Failed to transfer: ${error.message}`);
    },
    onSuccess: async (txHash: string) => {
      setMessage(`Blob tx sequenced, pending signature`);
      console.log("blob tx hash:", txHash);
      const signature = await signMessage(txHash);

      // Create proof
      const proof: Proof = {
        tx_hash: txHash,
        contract_name: idContractName,
        identity: account + "." + idContractName,
        signature: signature,
      };

      // Send proof transaction
      const responseProof = await fetch(`${HYLE_PROVER_URL}/prove`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(proof),
      });

      const generatedProof = await responseProof.text();
      console.log("generated proof:", generatedProof);

      setMessage(`Transfer successful`);
    },
  });


  return (
    <form onSubmit={handleSubmit}>
      <TokenSelector token={token} onTokenChange={setToken} />

      <Input
        type="text"
        labelText="From"
        name="account"
        value={account}
        suffixText=""
        readOnly
      />
      <Input
        type="text"
        name="recipient"
        labelText="To"
        value={recipient}
        suffixText=""
        onChange={(e) => setRecipient(e.target.value)}
      />
      <Input
        type="number"
        name="amount"
        labelText="Amount"
        placeholder="0"
        onChange={(e) => setAmount(Number(e.target.value))}
      />

      <p>{`Balance: ${getBalance(account) || `0`}`}</p>

      <Button type="submit">
        {`Transfer ${amount} ${token}`}
      </Button>

      <p>{message}</p>
    </form>
  );
}

