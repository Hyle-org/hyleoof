import { Blob } from "hyle";
import { useInvokeSnap } from "./useInvokeSnap";
import { signMessage } from "@/utils/sign";

export type InvokeSnapParams = {
  blobs: Array<Blob>;
};

/**
 * Utility hook to wrap the `sign_blobs` snap rpc call.
 */
export const useSignBlobs = (useSnap = false) => {
  const invokeSnap = useInvokeSnap();

  const signBlobsWithSnap = async ({ blobs }: InvokeSnapParams) =>
    invokeSnap({
      method: "sign_blobs",
      params: {
        blobs,
      },
    }) as Promise<{ signature: string; account: string; nonce: number }>;

  //// Sign message using personal_sign
  const signBlobs = async ({ blobs }: { blobs: Array<Blob> }) => {
    const nonce = new Date().getTime();

    console.log(blobs);
    console.log(nonce);

    const message = `verify ${nonce} ${blobs.map((blob) => blob.contract_name + " [" + blob.data.join(", ") + "]").join(" ")}`;

    const { signature, account } = await signMessage(message);

    return { account, signature, nonce };
  };

  return useSnap ? signBlobsWithSnap : signBlobs;
};
