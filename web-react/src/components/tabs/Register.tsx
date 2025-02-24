import { FormEvent, useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import { useMetaMask } from "@/hooks";
import { Blob, blob_builder, BlobTransaction } from "hyle";
import { signMessage } from "@/utils/sign";
import * as node from "@/api/node";
import { useFetchEvents } from "@/hooks/useFetchEvents";
import { useNotification } from "@/hooks/NotificationContext";

export default function Register() {
  const { account } = useMetaMask();
  const [message, setMessage] = useState("");
  const { addNotification } = useNotification();
  const fetchEvents = useFetchEvents(addNotification, () => {
    setMessage("Your identity is now registered on-chain, you can initiate transfer.");
  });

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    try {
      setMessage("Pending signature...");

      const { signature, account } = await signMessage("hyle registration");

      console.log('signature', signature);

      const register: Blob = blob_builder.metamask.register(signature);

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
