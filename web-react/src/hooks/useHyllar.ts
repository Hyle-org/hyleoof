import { getContractState, GetContractStateResponse } from "@/api/indexer/getContractState";
import { camelizeKeys, CamelizeKeys } from "@/utils/camelizeKeys";
import { useEffect, useState } from "react";

interface HyllarToken extends CamelizeKeys<GetContractStateResponse> {}

export function useHyllar() {
  const [hyllarState, setHyllarState] = useState<HyllarToken | null>();

  const setHyllar = async () => {
    const response = await getContractState({ contractName: "hyllar" });
    const camelizedResponse = camelizeKeys(response);
    setHyllarState(camelizedResponse);
  }

  useEffect(() => {
    setHyllar();
  }, []);

  return { hyllarState };
}
