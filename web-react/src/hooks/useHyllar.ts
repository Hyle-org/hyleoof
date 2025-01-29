import { getContractState, GetContractStateResponse } from "@/api/indexer/getContractState";
import { camelizeKeys, CamelizeKeys } from "@/utils/camelizeKeys";
import { useEffect, useState } from "react";

interface HyllarToken extends CamelizeKeys<GetContractStateResponse> {
  balances: { [key: string]: number };
}

interface UseHyllarParams {
  contractName: string;
}

/**
 * Custom hook to manage and fetch Hyllar token contract state
 * @param {Object} params - Hook parameters
 * @param {string} params.contractName - Name of the contract to fetch state for
 * @returns {Object} Hook state and utility functions
 * @returns {HyllarToken|null} returns.hyllarState - Current state of the Hyllar contract
 * @returns {(hydentity: string) => number|undefined} returns.getHydentityBalance - Function to get balance for a specific hydentity
 * @returns {() => number|undefined} returns.getTotalSupply - Function to get the total token supply
 */
export function useHyllar({ contractName }: UseHyllarParams) {
  const [hyllarState, setHyllarState] = useState<HyllarToken | null>(null);

  const setHyllar = async () => {
    try {
      const response = await getContractState({ contractName });
      const camelizedResponse = camelizeKeys(response);
      setHyllarState(camelizedResponse);
    } catch (error) {
      console.error("Failed to fetch contract state:", error);
    }
  };

  const getHydentityBalance = (hydentity: string) => {
    const balance = hyllarState?.balances[hydentity];
    return balance;
  }

  const getTotalSupply = () => hyllarState?.totalSupply;

  useEffect(() => {
    setHyllar();
  }, [contractName]);

  return { hyllarState, getHydentityBalance, getTotalSupply };
}
