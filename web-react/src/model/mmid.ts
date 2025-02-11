import { idContractName } from "@/config";
import { borshSerialize, BorshSchema } from "borsher";
import { Blob } from "@/model/hyle";

export type IdentityAction =
  | {
      RegisterIdentity: {
        account: string;
      };
    }
  | {
      VerifyIdentity: {
        account: string;
        signature: string;
        nonce: number;
      };
    };

export const buildIdentityBlob = (
  account: string,
  nonce: number,
  signature: string,
): Blob => {
  const action: IdentityAction = {
    VerifyIdentity: { account, signature, nonce },
  };

  const blob: Blob = {
    contract_name: idContractName,
    data: serializeIdentityAction(action),
  };
  return blob;
};

const serializeIdentityAction = (action: IdentityAction): number[] => {
  return Array.from(borshSerialize(schema, action));
};

const schema = BorshSchema.Enum({
  RegisterIdentity: BorshSchema.Struct({
    account: BorshSchema.String,
  }),
  VerifyIdentity: BorshSchema.Struct({
    account: BorshSchema.String,
    nonce: BorshSchema.u32,
  }),
});
