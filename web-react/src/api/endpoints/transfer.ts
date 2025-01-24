import { createApiRequest } from "../createApiRequest";
import { AuthParams } from "../common";

interface TransferParams extends AuthParams {
  recipient: string;
  token: string;
  amount: number;
}

export default async function transfer({
  username,
  password,
  recipient,
  token,
  amount,
}: TransferParams) {
  return createApiRequest({
    endpoint: "/transfer",
    method: "POST",
    body: {
      username,
      password,
      recipient,
      token,
      amount,
    },
  })();
}
