import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";

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
    baseUrl: SERVER_URL,
    endpoint: "/swap",
    method: "POST",
    body: {
      username: username + ".hydentity",
      password,
      token_a: fromToken,
      token_b: toToken,
      amount: Number(amount),
    },
  })();
}
