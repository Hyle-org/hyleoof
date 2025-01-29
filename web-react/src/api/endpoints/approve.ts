import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";

interface ApproveParams extends AuthParams {
  spender: string;
  token: string;
  amount: number;
}

export default async function approve({
  username,
  password,
  spender,
  token,
  amount,
}: ApproveParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/approve",
    method: "POST",
    body: {
      username,
      password,
      token,
      spender,
      amount: Number(amount),
    },
  })();
}
