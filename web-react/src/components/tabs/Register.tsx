import { FormEvent, useState } from "react";
import Button from "@/components/ui/Button";
import Input from "@/components/ui/Input";
import { useMutation } from "@tanstack/react-query";
import register from "@/api/endpoints/register";

type Progress = "ready" | "registering" | "success" | "failed";

export default function Register() {
  const mutation = useMutation({
    mutationFn: function (formData: FormData) {
      return register({
        username: formData.get("username")?.toString() || "",
        password: formData.get("password")?.toString() || "",
      });
    },
    onMutate: function (formData: FormData) {
      setProgress("registering");
    },
    onError: function (error) {
      setProgress("failed");
    },
    onSuccess: function (data) {
      setProgress("success");
    },
  });

  const [username, setUsername] = useState("");
  const [progress, setProgress] = useState<Progress>("ready");

  function handleRegister(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    // setProgress("registering");
    mutation.mutate(new FormData(event.currentTarget));
  }

  return (
    <form onSubmit={handleRegister}>
      <Input
        type="text"
        labelText="Username"
        suffixText=".hydentity"
        value={username}
        name="hydentity"
        onChange={(e) => setUsername(e.target.value)}
      />
      <Input type="password" labelText="Password" name="password" />

      <p>{`Progress: ${progress}`}</p>
      <Button>{`Register ${username}.hydentity`}</Button>
    </form>
  );
}
