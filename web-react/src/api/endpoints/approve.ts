import { createApiRequest } from "../createApiRequest";
import { AuthParams } from "../common";

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
