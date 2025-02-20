import { FormEvent, useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import { useMetaMask } from "@/hooks";
import { Blob, BlobTransaction } from "@/model/hyle";
import { signMessage } from "@/utils/sign";
import { buildRegisterBlob } from "@/model/mmid";
import * as node from "@/api/node";
import { useFetchEvents } from "@/hooks/useFetchEvents";

export default function Register() {
  const { account } = useMetaMask();
  const [message, setMessage] = useState("");
  const fetchEvents = useFetchEvents(setMessage, () => {
    setTimeout(() => setMessage("Your identity is now registered on-chain, you can initiate transfer."), 2000);
  });

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    try {
      setMessage("Pending signature...");

      const { signature, account } = await signMessage("hyle registration");

      console.log('signature', signature);

      const register: Blob = buildRegisterBlob(signature);

      const blobTx: BlobTransaction = {
        identity: account,
        blobs: [register],
      };

      console.log('blob', blobTx);

      const tx = await node.sendBlobTx(blobTx);
      fetchEvents(tx);

      setMessage("Transaction sent ✅");
    } catch (error) {
      const err = error as Error;
      setMessage(`Failed to register: ${err.message}`);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <Input
        type="text"
        labelText="Account"
        suffixText=""
        value={account}
        name="account"
        readOnly
      />

      <Button>{`Register`}</Button>
      <p>{message}</p>
    </form>
  );
}
