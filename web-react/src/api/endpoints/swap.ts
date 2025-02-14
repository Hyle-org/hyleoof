import { AuthParams } from "../constants";
import {
  buildApproveBlob,
  buildTransferBlob,
  buildTransferFromBlob,
} from "@/model/token";
import { Blob } from "@/model/hyle";
import { buildSwapBlob } from "@/model/amm";

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
  // Blob 0 is identity
  // Blob 1
  const allow: Blob = buildApproveBlob(fromToken, "amm", amount_a);

  // Blob 2
  const swap: Blob = buildSwapBlob(
    fromToken,
    toToken,
    amount_a,
    amount_b,
    [3, 4],
  );

  // Blob 3
  const transferFrom: Blob = buildTransferFromBlob(
    account,
    "amm",
    fromToken,
    amount_a,
    2,
  );

  // Blob 4
  const transfer: Blob = buildTransferBlob(account, toToken, amount_b, 2);

  return [allow, swap, transferFrom, transfer];
}
