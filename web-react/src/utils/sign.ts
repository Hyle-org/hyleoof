import { getSnapsProvider } from "./metamask";

export async function signMessage(message: string) {
  const ethereum = await getSnapsProvider();
  const hexMessage = toHexMessage(message); // Convert message to hex
  console.log(hexMessage);
  const ethAddr = await ethereum.request({
    method: "eth_requestAccounts",
  });
  console.log(ethAddr[0]);

  const signature = await ethereum.request<string>({
    method: "personal_sign",
    params: [hexMessage, ethAddr[0]],
  });

  return signature;
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
