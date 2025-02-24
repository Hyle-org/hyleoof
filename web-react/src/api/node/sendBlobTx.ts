import { BlobTransaction } from "hyle";
import { NODE_URL } from "../constants";
import { createApiRequest } from "../createApiRequest";

export async function sendBlobTx(tx: BlobTransaction) {
  return createApiRequest<string>({
    baseUrl: NODE_URL,
    endpoint: `/v1/tx/send/blob`,
    method: "POST",
    body: tx,
  })();
}
