import { borshSerialize, BorshSchema } from "borsher";
import { ContractName, Blob } from "./hyle";

//pub struct StructuredBlobData<Parameters> {
//    pub caller: Option<BlobIndex>,
//    pub callees: Option<Vec<BlobIndex>>,
//    pub parameters: Parameters,
//}
export type StructuredBlobData<Parameters> = {
  caller: number | null;
  callees: number[] | null;
  parameters: Parameters;
};

const structuredBlobDataSchema = (schema: BorshSchema) =>
  BorshSchema.Struct({
    caller: BorshSchema.Option(BorshSchema.u32),
    callees: BorshSchema.Option(BorshSchema.Vec(BorshSchema.u32)),
    parameters: schema,
  });

//pub enum ERC20Action {
//    TotalSupply,
//    BalanceOf {
//        account: String,
//    },
//    Transfer {
//        recipient: String,
//        amount: u128,
//    },
//    TransferFrom {
//        sender: String,
//        recipient: String,
//        amount: u128,
//    },
//    Approve {
//        spender: String,
//        amount: u128,
//    },
//    Allowance {
//        owner: String,
//        spender: String,
//    },
//}

export type ERC20Action =
  | { TotalSupply: {} }
  | { BalanceOf: { account: string } }
  | { Transfer: { recipient: string; amount: number } }
  | { TransferFrom: { sender: string; recipient: string; amount: number } }
  | { Approve: { spender: string; amount: number } }
  | { Allowance: { owner: string; spender: string } };

export const buildTransferBlob = (
  recipient: string,
  token: ContractName,
  amount: number,
): Blob => {
  const action: ERC20Action = {
    Transfer: { recipient, amount },
  };

  const structured: StructuredBlobData<ERC20Action> = {
    caller: null,
    callees: null,
    parameters: action,
  };

  const blob: Blob = {
    contract_name: token,
    data: serializeERC20Action(structured),
  };
  return blob;
};

export const serializeERC20Action = (
  action: StructuredBlobData<ERC20Action>,
): number[] => {
  return Array.from(
    borshSerialize(structuredBlobDataSchema(erc20Schema), action),
  );
};

const erc20Schema = BorshSchema.Enum({
  TotalSupply: BorshSchema.Unit,

  BalanceOf: BorshSchema.Struct({
    account: BorshSchema.String,
  }),

  Transfer: BorshSchema.Struct({
    recipient: BorshSchema.String,
    amount: BorshSchema.u128,
  }),

  TransferFrom: BorshSchema.Struct({
    sender: BorshSchema.String,
    recipient: BorshSchema.String,
    amount: BorshSchema.u128,
  }),

  Approve: BorshSchema.Struct({
    spender: BorshSchema.String,
    amount: BorshSchema.u128,
  }),

  Allowance: BorshSchema.Struct({
    owner: BorshSchema.String,
    spender: BorshSchema.String,
  }),
});
