import { idContractName } from "@/config";
import { borshSerialize, BorshSchema } from "borsher";
import { Blob } from "@/model/hyle";

export type IdentityAction =
  | {
      RegisterIdentity: {
        signature: string;
      };
    }
  | {
      VerifyIdentity: {
        nonce: number;
        signature: string;
      };
    };

export const buildRegisterBlob = (signature: string): Blob => {
  const action: IdentityAction = {
    RegisterIdentity: { signature },
  };
  const blob: Blob = {
    contract_name: idContractName,
    data: serializeIdentityAction(action),
  };
  return blob;
};

export const buildIdentityBlob = (nonce: number, signature: string): Blob => {
  const action: IdentityAction = {
    VerifyIdentity: { nonce, signature },
  };

  console.log("action", action);

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
    signature: BorshSchema.String,
  }),
  VerifyIdentity: BorshSchema.Struct({
    nonce: BorshSchema.u128,
    signature: BorshSchema.String,
  }),
});
