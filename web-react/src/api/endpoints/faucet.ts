import { idContractName } from "@/config/contract";
import { SERVER_URL } from "../constants";
import { createApiRequest } from "../createApiRequest";

interface FaucetParams {
  account: string;
  token: string;
}

export default async function faucet({ account, token }: FaucetParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/faucet",
    method: "POST",
    body: {
      account: account,
      token,
    },
  })();
}
