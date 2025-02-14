import { borshSerialize, BorshSchema } from "borsher";
import { ContractName, Blob } from "./hyle";
import {
  BlobIndex,
  StructuredBlobData,
  structuredBlobDataSchema,
} from "./token";

// pub enum AmmAction {
//    Swap {
//        pair: TokenPair, // User swaps the first token of the pair for the second token
//        amounts: TokenPairAmount,
//    },
//    NewPair {
//        pair: TokenPair,
//        amounts: TokenPairAmount,
//    },
//}
//
// type TokenPair = (String, String);
// type TokenPairAmount = (u128, u128);

export type TokenPair = [string, string];
export type TokenPairAmount = [number, number];

export type AmmAction =
  | {
      Swap: {
        pair: TokenPair;
        amounts: TokenPairAmount;
      };
    }
  | {
      NewPair: {
        pair: TokenPair;
        amounts: TokenPairAmount;
      };
    };

const ammSchema = BorshSchema.Enum({
  Swap: BorshSchema.Struct({
    pair: BorshSchema.Struct({ 0: BorshSchema.String, 1: BorshSchema.String }),
    amounts: BorshSchema.Struct({ 0: BorshSchema.u128, 1: BorshSchema.u128 }),
  }),
  NewPair: BorshSchema.Struct({
    pair: BorshSchema.Struct({ 0: BorshSchema.String, 1: BorshSchema.String }),
    amounts: BorshSchema.Struct({ 0: BorshSchema.u128, 1: BorshSchema.u128 }),
  }),
});

export const buildSwapBlob = (
  token_a: ContractName,
  token_b: ContractName,
  amount_a: number,
  amount_b: number,
  callees: number[] | null,
): Blob => {
  const action: AmmAction = {
    Swap: { pair: [token_a, token_b], amounts: [amount_a, amount_b] },
  };

  const structured: StructuredBlobData<AmmAction> = {
    caller: null,
    callees: callees ? callees.map((c): BlobIndex => ({ 0: c })) : null,
    parameters: action,
  };

  const blob: Blob = {
    contract_name: "amm",
    data: serializeAmmAction(structured),
  };
  return blob;
};

export const serializeAmmAction = (
  action: StructuredBlobData<AmmAction>,
): number[] => {
  return Array.from(
    borshSerialize(structuredBlobDataSchema(ammSchema), action),
  );
};
