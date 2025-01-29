import { createApiRequest } from "../createApiRequest";
import { AuthParams, SERVER_URL } from "../constants";

interface RegisterParams extends AuthParams {}

export default async function register({ username, password }: RegisterParams) {
  return createApiRequest({
    baseUrl: SERVER_URL,
    endpoint: "/register",
    method: "POST",
    body: {
      username,
      password,
    },
  })();
}
