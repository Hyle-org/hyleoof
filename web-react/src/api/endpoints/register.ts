import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";
import { idContractName } from "@/config/contract";

interface RegisterParams extends AuthParams {}

export default async function register({ account }: RegisterParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/register",
    method: "POST",
    body: {
      account: account + "." + idContractName,
    },
  })();
}
