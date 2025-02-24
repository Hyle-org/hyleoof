import { AuthParams } from "../constants";
import { blob_builder } from "hyle";

interface SwapParams extends AuthParams {
  fromToken: string;
  toToken: string;
  amount_a: number;
  amount_b: number;
}

export default async function swap({
  account,
  fromToken,
  toToken,
  amount_a,
  amount_b,
}: SwapParams) {
  return blob_builder.amm.swap(account, fromToken, toToken, amount_a, amount_b);
}
