import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";

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
    baseUrl: SERVER_URL,
    endpoint: "/transfer",
    method: "POST",
    body: {
      username,
      password,
      recipient,
      token,
      amount: Number(amount),
    },
  })();
}
