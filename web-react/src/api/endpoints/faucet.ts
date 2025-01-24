import { createApiRequest } from "../createApiRequest";

interface FaucetParams {
  username: string;
  token: string;
}

export default async function faucet({ username, token }: FaucetParams) {
  return createApiRequest({
    endpoint: "/faucet",
    method: "POST",
    body: {
      username,
      token,
    },
  })();
}
