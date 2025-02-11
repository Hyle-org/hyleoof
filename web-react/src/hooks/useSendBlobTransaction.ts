import { signMessage } from "@/utils/sign";
import { Blob, BlobTransaction } from "@/model/hyle";
import { buildIdentityBlob } from "@/model/mmid";
import * as node from "@/api/node";

export const useSendBlobTransaction = () => {
  /**
   * Request the Snap.
   */
  const sendBlobTx = async (blobs: Array<Blob>) => {
    const { signature, account, nonce } = await signMessage(blobs);

    const verifyIdentity: Blob = buildIdentityBlob(account, nonce, signature);

    const blobTx: BlobTransaction = {
      identity: account,
      blobs: [verifyIdentity, ...blobs],
    };

    await node.sendBlobTx(blobTx);
  };

  return sendBlobTx;
};
