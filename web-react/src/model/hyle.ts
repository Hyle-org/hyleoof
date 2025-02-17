export type ContractName = string;
export type Identity = string;

export interface Blob {
  contract_name: ContractName;
  data: number[];
}
export interface BlobTransaction {
  identity: Identity;
  blobs: Blob[];
}
