import { useEffect, useState } from "react";

import { defaultSnapOrigin } from "../config";
import type { GetSnapsResponse } from "../types";
import { useRequest } from "./useRequest";
import { useMetaMaskContext } from "./MetamaskContext";

/**
 * A Hook to retrieve useful data from MetaMask.
 * @returns The informations.
 */
export const useMetaMask = () => {
  const {
    provider,
    setInstalledSnap,
    installedSnap,
    account,
    setAccount,
    setNonce,
    useSnap,
    setUseSnap,
  } = useMetaMaskContext();
  const request = useRequest();

  const [isFlask, setIsFlask] = useState(false);

  const providerDetected = provider !== null;

  /**
   * Detect if the version of MetaMask is Flask.
   */
  const detectFlask = async () => {
    const clientVersion = await request({
      method: "web3_clientVersion",
    });

    const isFlaskDetected = (clientVersion as string[])?.includes("flask");

    setIsFlask(isFlaskDetected);
  };

  /**
   * Get the Snap informations from MetaMask.
   */
  const getSnap = async () => {
    const snaps = (await request({
      method: "wallet_getSnaps",
    })) as GetSnapsResponse;

    setInstalledSnap(snaps[defaultSnapOrigin] ?? null);
  };

  useEffect(() => {
    const detect = async () => {
      if (provider) {
        await detectFlask();
        await getSnap();
      }
    };

    detect().catch(console.error);
  }, [provider]);

  return {
    isFlask,
    providerDetected,
    provider,
    installedSnap,
    getSnap,
    account,
    setAccount,
    setNonce,
    useSnap,
    setUseSnap,
  };
};
