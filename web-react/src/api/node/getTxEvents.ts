import { TxEvent } from "@/model/hyle";
import { NODE_URL } from "../constants";
import { createApiRequest } from "../createApiRequest";

export async function getTxEvents(tx: string) {
  return createApiRequest<TxEvent[]>({
    baseUrl: NODE_URL,
    endpoint: `/v1/indexer/transaction/hash/${tx}/events`,
    method: "GET",
  })();
}
