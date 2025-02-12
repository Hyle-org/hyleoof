import { Blob } from "@/model/hyle";
import { getSnapsProvider } from "./metamask";
import { idContractName } from "@/config";

export async function signMessage(message: string) {
  const hexMessage = toHexMessage(message);

  const ethereum = await getSnapsProvider();
  const ethAddr = await ethereum.request({
    method: "eth_requestAccounts",
  });

  const signature = await ethereum.request<string>({
    method: "personal_sign",
    params: [hexMessage, ethAddr[0]],
  });

  const account = `${ethAddr[0]}.${idContractName}`;
  return { signature, account };
}

export async function signBlobs(blobs: Array<Blob>) {
  // TODO: fetch nonce
  const nonce = 0;

  const message = `verify ${nonce} ${blobs.map((blob) => blob.contract_name + " [" + blob.data.join(", ") + "]").join(" ")}`;

  const { signature, account } = await signMessage(message);

  return { signature, account, nonce };
}

// Convert message to hex format
function toHexMessage(message: string): string {
  return (
    "0x" +
    message
      .split("")
      .map((char) => char.charCodeAt(0).toString(16))
      .join("")
  );
}
