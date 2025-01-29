import { expect, test, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import createFetchMock from "vitest-fetch-mock";
import { GetContractStateResponse } from "@/api/indexer/getContractState";
import { useHyllar } from "@/hooks/useHyllar";
import { camelizeKeys } from "@/utils/camelizeKeys";
import { NODE_URL } from "@/api/constants";

const fetchMocker = createFetchMock(vi);
fetchMocker.enableMocks();

const mockGetContractStateResponse: GetContractStateResponse = {
    total_supply: 1000,
    balances: {
        "alice.hydentity": 100,
        "bob.hydentity": 200,
    },
    allowances: [[50, ["alice.hydentity", "bob.hydentity"]]],
};


test("to call the Hylé Indexer and return the state of the Hyllar contract", async () => {
    fetchMocker.mockResponseOnce(JSON.stringify(mockGetContractStateResponse));
    const { result } = renderHook(() => useHyllar({ contractName: "hyllar" }));
    await waitFor(() => {
        expect(result.current.hyllarState)
            .toEqual(camelizeKeys(mockGetContractStateResponse))
    });
    expect(fetchMocker).toBeCalledWith(`${NODE_URL}/v1/indexer/contract/hyllar/state`, {
        headers: { "Content-Type": "application/json" },
        method: "GET",
    })
});