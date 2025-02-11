import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";
import { idContractName } from "@/config/contract";

interface TransferParams extends AuthParams {
  recipient: string;
  token: string;
  amount: number;
}

export default async function transfer({
  account,
  recipient,
  token,
  amount,
}: TransferParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/transfer",
    method: "POST",
    body: {
      account: account + "." + idContractName,
      recipient,
      token,
      amount: Number(amount),
    },
  })();
}
