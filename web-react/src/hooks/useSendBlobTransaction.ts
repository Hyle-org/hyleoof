import { Blob, BlobTransaction } from "@/model/hyle";
import { buildIdentityBlob } from "@/model/mmid";
import * as node from "@/api/node";

export const useSendBlobTransaction = () => {
  /**
   * Request the Snap.
   */
  const sendBlobTx = async (
    blobs: Array<Blob>,
    account: string,
    nonce: number,
    signature: string,
  ) => {
    const verifyIdentity: Blob = buildIdentityBlob(nonce, signature);

    const blobTx: BlobTransaction = {
      identity: account,
      blobs: [verifyIdentity, ...blobs],
    };

    return await node.sendBlobTx(blobTx);
  };

  return sendBlobTx;
};
