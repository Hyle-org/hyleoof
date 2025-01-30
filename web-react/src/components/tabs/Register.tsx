import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import register from "@/api/endpoints/register";
import { useFormSubmission } from "@/hooks/useFormSubmission";

export default function Register() {
  const [username, setUsername] = useState("");
  const [message, setMessage] = useState("");

  const { handleSubmit } = useFormSubmission(register, {
    onMutate: () => {
      setMessage("Registering...");
    },
    onError: (error) => {
      setMessage(`Failed to register: ${error.message}`);
    },
    onSuccess: () => {
      setMessage(`Register successful for user ${username}.hydentity`);
    },
  });

  return (
    <form onSubmit={handleSubmit}>
      <Input
        type="text"
        labelText="Username"
        suffixText=".hydentity"
        value={username}
        name="username"
        onChange={(e) => setUsername(e.target.value)}
      />
      <Input type="password" labelText="Password" name="password" />

      
      <Button>{`Register ${username}.hydentity`}</Button>
      <p>{message}</p>
    </form>
  );
}
