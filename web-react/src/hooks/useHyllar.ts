import {
  getContractState,
  GetContractStateResponse,
} from "@/api/indexer/getContractState";
import { idContractName } from "@/config/contract";
import { camelizeKeys, CamelizeKeys, toCamelCase } from "@/utils/camelizeKeys";
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
 * @returns {(account: string) => number|undefined} returns.getBalance - Function to get balance for a specific identity
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

  const getBalance = (account: string) => {
    const id = toCamelCase(account);
    const balance = hyllarState?.balances[id];
    return balance;
  };

  const getTotalSupply = () => hyllarState?.totalSupply;

  useEffect(() => {
    setHyllar();
  }, [contractName]);

  return { hyllarState, getBalance, getTotalSupply };
}
