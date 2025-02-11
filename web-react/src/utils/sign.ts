import { Blob } from "@/model/hyle";
import { getSnapsProvider } from "./metamask";
import { idContractName } from "@/config";

export async function signMessage(blobs: Array<Blob>) {
  // TODO: fetch nonce
  const message = {
    nonce: 0,
    blobs,
  };
  const ethereum = await getSnapsProvider();
  const hexMessage = toHexMessage(JSON.stringify(message));
  const ethAddr = await ethereum.request({
    method: "eth_requestAccounts",
  });
  console.log(ethAddr[0]);

  const signature = await ethereum.request<string>({
    method: "personal_sign",
    params: [hexMessage, ethAddr[0]],
  });

  const account = `${ethAddr[0]}.${idContractName}`;
  const { nonce } = message;

  return { signature, account, nonce };
}
// Convert message to hex format
export function toHexMessage(message: string): string {
  var hex, i;

  var result = "0x";
  for (i = 0; i < message.length; i++) {
    hex = message.charCodeAt(i).toString(16);
    result += ("000" + hex).slice(-4);
  }

  return result;
}
