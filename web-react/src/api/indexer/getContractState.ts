import { NODE_URL } from "../constants";
import { createApiRequest } from "../createApiRequest";

interface GetContractStateParams {
  contractName: string;
}

export interface GetContractStateResponse {
  total_supply: number;
  balances: { [key: string]: number };
  allowances: [[ number, [string]]];
}

export async function getContractState({ contractName }: GetContractStateParams) {
  return createApiRequest<GetContractStateResponse>({
    baseUrl: NODE_URL,
    endpoint: `/v1/indexer/contract/${contractName}/state`,
    method: "GET",
  })();
}
