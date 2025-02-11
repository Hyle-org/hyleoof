import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";
import { idContractName } from "@/config/contract";

interface SwapParams extends AuthParams {
  fromToken: string;
  toToken: string;
  amount: number;
}

export default async function swap({
  account,
  fromToken,
  toToken,
  amount,
}: SwapParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/swap",
    method: "POST",
    body: {
      account: account + "." + idContractName,
      token_a: fromToken,
      token_b: toToken,
      amount: Number(amount),
    },
  })();
}
