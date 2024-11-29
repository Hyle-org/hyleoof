use anyhow::{bail, Error, Result};
use borsh::to_vec;
use hyle::{
    indexer::model::ContractDb,
    model::{BlobTransaction, ProofData, ProofTransaction},
    rest::client::ApiHttpClient,
};
use sdk::{
    Blob, ContractInput, ContractName, Digestable, HyleOutput, Identity, StateDigest, TxHash,
};
use tracing::info;

pub async fn run<State, Builder>(
    client: &ApiHttpClient,
    contract_name: &ContractName,
    binary: &'static [u8],
    build_contract_input: Builder,
) -> Result<ProofData>
where
    State: TryFrom<sdk::StateDigest, Error = Error>,
    State: Digestable + std::fmt::Debug + serde::Serialize,
    Builder: Fn(State) -> ContractInput<State>,
{
    let initial_state = fetch_current_state(client, contract_name).await?;
    info!("Fetched current state: {:?}", initial_state);

    let contract_input = build_contract_input(initial_state);

    info!("{}", "-".repeat(20));
    info!("Checking transition for {contract_name}...");
    let execute_info = execute(binary, &contract_input)?;
    let output = execute_info.journal.decode::<HyleOutput>().unwrap();
    if !output.success {
        let program_error = std::str::from_utf8(&output.program_outputs).unwrap();
        bail!(
            "\x1b[91mExecution failed ! Program output: {}\x1b[0m",
            program_error
        );
    } else {
        let next_state: State = output.next_state.try_into().unwrap();
        info!("New state: {:?}", next_state);
    }

    info!("{}", "-".repeat(20));
    info!("Proving transition for {contract_name}...");

    let input = bonsai_runner::as_input_data(&contract_input)?;

    let receipt = bonsai_runner::run_bonsai(binary, input).await?;

    let hyle_output = receipt
        .journal
        .decode::<HyleOutput>()
        .expect("Failed to decode journal");

    if !hyle_output.success {
        let program_error = std::str::from_utf8(&hyle_output.program_outputs).unwrap();
        bail!(
            "\x1b[91mExecution failed ! Program output: {}\x1b[0m",
            program_error
        );
    }

    let encoded_receipt = to_vec(&receipt).expect("Unable to encode receipt");
    Ok(ProofData::Bytes(encoded_receipt))
}

pub async fn fetch_current_state<State>(
    client: &ApiHttpClient,
    contract_name: &ContractName,
) -> Result<State, Error>
where
    State: TryFrom<sdk::StateDigest, Error = Error>,
{
    let resp = client
        .get_indexer_contract(contract_name)
        .await?
        .json::<ContractDb>()
        .await?;

    StateDigest(resp.state_digest).try_into()
}

fn execute<State>(
    binary: &'static [u8],
    contract_input: &ContractInput<State>,
) -> Result<risc0_zkvm::SessionInfo>
where
    State: Digestable + serde::Serialize,
{
    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(contract_input)
        .unwrap()
        .build()
        .unwrap();

    let prover = risc0_zkvm::default_executor();
    Ok(prover.execute(env, binary).unwrap())
}

fn prove<State>(
    binary: &'static [u8],
    contract_input: &ContractInput<State>,
) -> Result<risc0_zkvm::ProveInfo>
where
    State: Digestable + serde::Serialize,
{
    let env = risc0_zkvm::ExecutorEnv::builder()
        .write(contract_input)
        .unwrap()
        .build()
        .unwrap();

    let prover = risc0_zkvm::default_prover();
    Ok(prover.prove(env, binary).unwrap())
}

pub async fn send_proof(
    client: &ApiHttpClient,
    blob_tx_hash: TxHash,
    contract_name: ContractName,
    proof: ProofData,
) -> Result<()> {
    let res = client
        .send_tx_proof(&ProofTransaction {
            blob_tx_hash,
            contract_name,
            proof,
        })
        .await?;
    assert!(res.status().is_success());

    info!("Proof sent successfully");
    info!("Response: {}", res.text().await?);

    Ok(())
}

pub async fn send_blobs(
    client: &ApiHttpClient,
    identity: Identity,
    blobs: Vec<Blob>,
) -> Result<TxHash> {
    let tx_hash = client
        .send_tx_blob(&BlobTransaction { identity, blobs })
        .await?;

    info!("Blob sent successfully. Response: {}", tx_hash);

    Ok(tx_hash)
}
