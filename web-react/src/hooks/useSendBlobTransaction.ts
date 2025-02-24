import { Blob, blob_builder, BlobTransaction } from "hyle";
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
    const verifyIdentity: Blob = blob_builder.metamask.verifyIdentity(
      nonce,
      signature,
    );

    const blobTx: BlobTransaction = {
      identity: account,
      blobs: [verifyIdentity, ...blobs],
    };

    return await node.sendBlobTx(blobTx);
  };

  return sendBlobTx;
};
