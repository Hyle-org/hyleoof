import TokenSelector from "@/components/TokenSelector";
import Input from "@/components/ui/Input";
import Button from "../ui/Button";
import { useState } from "react";
import swap from "@/api/endpoints/swap";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useHyllar } from "@/hooks/useHyllar";
import { useMetaMask } from "@/hooks";
import { createApiRequest } from "@/api/createApiRequest";
import { SERVER_URL } from "@/api/constants";
import { Blob } from "@/model/hyle";
import { useSendBlobTransaction } from "@/hooks/useSendBlobTransaction";
import { useSignBlobs } from "@/hooks/useSignBlobs";

export default function Swap() {
  const { account } = useMetaMask();
  const [fromToken, setFromToken] = useState("hyllar");
  const [toToken, setToToken] = useState("hyllar2");
  const [fromTokenAmount, setFromTokenAmount] = useState(0);
  const [toTokenAmount, setToTokenAmount] = useState(0);
  const [message, setMessage] = useState("");
  const { getBalance } = useHyllar({ contractName: fromToken });
  const sendBlobTransaction = useSendBlobTransaction();
  const signBlobs = useSignBlobs();

  const setAmount = async (value: number) => {
    setFromTokenAmount(value);
    const amount_b = (await createApiRequest({
      baseUrl: SERVER_URL,
      endpoint: `/paired_amount/${fromToken}/${toToken}/${value}`,
      method: "GET",
    })()) as number;
    setToTokenAmount(amount_b);
  }

  const { handleSubmit } = useFormSubmission(swap, {
    onMutate: () => {
      setMessage("Swaping...");
    },
    onError: (error) => {
      setMessage(`Failed to swap: ${error.message}`);
    },
    onSuccess: async (blobs: Array<Blob>) => {
      setMessage(`Pending signature`);

      const res = await signBlobs({ blobs });
      if (res == null) {
        throw new Error('Signature failed');
      }

      const { account, signature, nonce } = res;

      await sendBlobTransaction(blobs, account, nonce, signature);

      setMessage(`Transaction sent ✅`);
    },
  });

  return (
    <form onSubmit={handleSubmit}>
      <Input
        type="text"
        labelText="Account:"
        value={account}
        name="account"
        suffixText=""
        readOnly
      />

      <TokenSelector
        token={fromToken}
        name="fromToken"
        onTokenChange={setFromToken}
      />
      <TokenSelector
        token={toToken}
        name="toToken"
        onTokenChange={setToToken}
      />

      <Input
        type="number"
        labelText="Amount"
        name="amount_a"
        placeholder="0"
        onChange={(e) => setAmount(Number(e.target.value))}
      />
      <Input
        type="number"
        labelText="You will receive"
        name="amount_b"
        placeholder="0"
        value={toTokenAmount}
        readOnly
      />

      <p>{`Balance: ${getBalance(account) || `0`}`}</p>
      <Button type="submit">{`Swap ${fromTokenAmount} from ${fromToken} to ${toToken}`}</Button>
      <p>{message}</p>
    </form>
  );
}
