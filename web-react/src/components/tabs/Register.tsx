import { FormEvent, useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import register from "@/api/endpoints/register";
import { useFormSubmission } from "@/hooks/useFormSubmission";
import { useInvokeSnap, useMetaMask } from "@/hooks";

export default function Register() {
  const { account } = useMetaMask();
  const [message, setMessage] = useState("");
  const invokeSnap = useInvokeSnap();

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    await invokeSnap({
      method: "register_account"
    })
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
