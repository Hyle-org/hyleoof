import { AuthParams } from "../constants";
import { Blob } from "@/model/hyle";
import { buildTransferBlob } from "@/model/token";

interface TransferParams extends AuthParams {
  recipient: string;
  token: string;
  amount: number;
}

export default async function transfer({
  recipient,
  token,
  amount,
}: TransferParams): Promise<Array<Blob>> {
  const transfer: Blob = buildTransferBlob(recipient, token, amount, null);

  return [transfer];
}
