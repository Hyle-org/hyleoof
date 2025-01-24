import { createApiRequest } from "../createApiRequest";
import { AuthParams } from "../common";

interface RegisterParams extends AuthParams {}

export default async function register({ username, password }: RegisterParams) {
  return createApiRequest({
    endpoint: "/register",
    method: "POST",
    body: {
      username,
      password,
    },
  })();
}
