import { SERVER_URL } from "../constants";
import { createApiRequest } from "../createApiRequest";

interface FaucetParams {
  username: string;
  token: string;
}

export default async function faucet({ username, token }: FaucetParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/faucet",
    method: "POST",
    body: {
      username: username + ".hydentity",
      token,
    },
  })();
}
