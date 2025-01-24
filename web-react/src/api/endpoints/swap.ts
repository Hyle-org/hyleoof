import { createApiRequest } from "../createApiRequest";
import { AuthParams } from "../common";

interface SwapParams extends AuthParams {
  fromToken: string;
  toToken: string;
  amount: number;
}

export default async function swap({
  username,
  password,
  fromToken,
  toToken,
  amount,
}: SwapParams) {
  return createApiRequest({
    endpoint: "/swap",
    method: "POST",
    body: {
      username,
      password,
      token_a: fromToken,
      token_b: toToken,
      amount: Number(amount),
    },
  })();
}
