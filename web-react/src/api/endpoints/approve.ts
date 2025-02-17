import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";
import { idContractName } from "@/config/contract";

interface ApproveParams extends AuthParams {
  spender: string;
  token: string;
  amount: number;
}

export default async function approve({
  account,
  spender = "amm",
  token,
  amount,
}: ApproveParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/approve",
    method: "POST",
    body: {
      account: account + "." + idContractName,
      token,
      spender,
      amount: Number(amount),
    },
  })();
}
