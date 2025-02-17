import { Blob } from "@/model/hyle";
import { useInvokeSnap } from "./useInvokeSnap";

export type InvokeSnapParams = {
  blobs: Array<Blob>;
};

/**
 * Utility hook to wrap the `sign_blobs` snap rpc call.
 */
export const useSignBlobs = () => {
  const invokeSnap = useInvokeSnap();

  const signBlobs = async ({ blobs }: InvokeSnapParams) =>
    invokeSnap({
      method: "sign_blobs",
      params: {
        blobs,
      },
    }) as Promise<{ signature: string; account: string; nonce: number }>;

  return signBlobs;
};
