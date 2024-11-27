use anyhow::{bail, Error, Result};
use borsh::to_vec;
use hyle::{
    model::{BlobTransaction, ProofData, ProofTransaction},
    node_state::model::Contract,
    rest::client::ApiHttpClient,
};
use sdk::{Blob, ContractInput, ContractName, Digestable, HyleOutput, Identity, TxHash};

pub async fn run<State, Builder>(
    client: &ApiHttpClient,
    contract_name: &ContractName,
    build_contract_input: Builder,
) -> Result<ProofData>
where
    State: TryFrom<sdk::StateDigest, Error = Error>,
    State: Digestable + std::fmt::Debug + serde::Serialize,
    Builder: Fn(State) -> ContractInput<State>,
{
    let initial_state = fetch_current_state(client, contract_name).await?;
    println!("Fetched current state: {:?}", initial_state);

    let contract_input = build_contract_input(initial_state);

    println!("{}", "-".repeat(20));
    println!("Checking transition for {contract_name}...");
    let execute_info = execute(contract_name, &contract_input)?;
    let output = execute_info.journal.decode::<HyleOutput>().unwrap();
    if !output.success {
        let program_error = std::str::from_utf8(&output.program_outputs).unwrap();
        bail!(
            "\x1b[91mExecution failed ! Program output: {}\x1b[0m",
            program_error
        );
    } else {
        let next_state: State = output.next_state.try_into().unwrap();
        println!("New state: {:?}", next_state);
    }

    println!("{}", "-".repeat(20));
    println!("Proving transition for {contract_name}...");
    let prove_info = prove(contract_name, &contract_input)?;

    let receipt = prove_info.receipt;

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
        .json::<Contract>()
        .await?;

    resp.state.try_into()
}

fn execute<State>(
    contract_name: &ContractName,
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
    let file_path = format!("../hyle/contracts/{}/{}.img", contract_name, contract_name);
    if let Ok(binary) = std::fs::read(file_path.as_str()) {
        Ok(prover.execute(env, &binary).unwrap())
    } else {
        println!("Could not read ELF binary at {}.", file_path);
        println!("Please ensure that the ELF binary is built and located at the specified path.");
        println!("\x1b[93m--> Tip: Did you run build_contracts.sh ?\x1b[0m");
        bail!("Could not read ELF binary");
    }
}

fn prove<State>(
    contract_name: &ContractName,
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
    let file_path = format!("../hyle/contracts/{}/{}.img", contract_name, contract_name);
    if let Ok(binary) = std::fs::read(file_path.as_str()) {
        Ok(prover.prove(env, &binary).unwrap())
    } else {
        println!("Could not read ELF binary at {}.", file_path);
        println!("Please ensure that the ELF binary is built and located at the specified path.");
        println!("\x1b[93m--> Tip: Did you run build_contracts.sh ?\x1b[0m");
        bail!("Could not read ELF binary");
    }
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

    println!("Proof sent successfully");
    println!("Response: {}", res.text().await?);

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

    println!("Blob sent successfully. Response: {}", tx_hash);

    Ok(tx_hash)
}
