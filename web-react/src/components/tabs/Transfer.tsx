import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import Button from "@/components/ui/Button";
import { useState } from "react";
import transfer from "@/api/endpoints/transfer";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useHyllar } from "@/hooks/useHyllar";
import { useMetaMask } from "@/hooks";
import { Blob } from "@/model/hyle";
import { useSendBlobTransaction } from "@/hooks/useSendBlobTransaction";
import { useSignBlobs } from "@/hooks/useSignBlobs";
import { useFetchEvents } from "@/hooks/useFetchEvents";

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
  const sendBlobTransaction = useSendBlobTransaction();
  const signBlobs = useSignBlobs();
  const fetchEvents = useFetchEvents(setMessage, () => {
    setTimeout(() => setMessage(""), 2000);
  });

  const { handleSubmit } = useFormSubmission(transfer, {
    onMutate: () => {
      setMessage("Transferring...");
    },
    onError: (error) => {
      setMessage(`Failed to transfer: ${error.message}`);
    },
    onSuccess: async (blobs: Array<Blob>) => {
      setMessage(`Pending signature`);

      const res = await signBlobs({ blobs });
      if (res == null) {
        throw new Error('Signature failed');
      }
      const { account, signature, nonce } = res;

      const tx = await sendBlobTransaction(blobs, account, nonce, signature);
      fetchEvents(tx);

      setMessage(`Transaction sent ✅`);
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

