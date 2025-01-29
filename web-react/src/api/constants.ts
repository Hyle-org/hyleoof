export const SERVER_URL = import.meta.env.VITE_SERVER_URL;
export const NODE_URL = import.meta.env.VITE_NODE_URL;

export interface AuthParams {
  username: string;
  password: string;
}
