import { AuthParams } from "../constants";
import { blob_builder, Blob } from "hyle";

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
  const transfer: Blob = blob_builder.token.transfer(
    recipient,
    token,
    amount,
    null,
  );

  return [transfer];
}
