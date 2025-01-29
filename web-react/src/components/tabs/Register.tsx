import { useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import register from "@/api/endpoints/register";
import { useFormSubmission } from "@/hooks/useFormSubmission";

// type Progress = "ready" | "registering" | "success" | "failed";

export default function Register() {
  const [username, setUsername] = useState("");
  // const [progress, setProgress] = useState<Progress>("ready");
  const { handleSubmit } = useFormSubmission(register, {
    onError: (error) => {},
    onSuccess: (data) => {},
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

      {/* <p>{`Progress: ${progress}`}</p> */}
      <Button>{`Register ${username}.hydentity`}</Button>
    </form>
  );
}
