use std::env;

use anyhow::{bail, Error, Result};
use hyle::{
    indexer::model::ContractDb,
    model::{ProofData, ProofTransaction},
    rest::client::ApiHttpClient,
};
use reqwest::{Client, Url};
use sdk::{
    Blob, BlobData, BlobIndex, ContractInput, ContractName, Digestable, HyleOutput, Identity,
    StateDigest, TxHash,
};
use serde::Serialize;
use tracing::info;

static HYLLAR_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/hyllar/hyllar.img");
static HYDENTITY_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/hydentity/hydentity.img");
static AMM_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/amm/amm.img");

fn get_binary(contract_name: ContractName) -> Result<&'static [u8]> {
    match contract_name.0.as_str() {
        "hyllar" | "hyllar2" => Ok(HYLLAR_BIN),
        "hydentity" => Ok(HYDENTITY_BIN),
        "amm" => Ok(AMM_BIN),
        _ => bail!("contract {} not supported", contract_name),
    }
}

pub struct ContractRunner {
    client: ApiHttpClient,
    contract_name: ContractName,
    contract_input: Vec<u8>,
}

impl ContractRunner {
    pub async fn new<State>(
        contract_name: ContractName,
        identity: Identity,
        private_blob: BlobData,
        blobs: Vec<Blob>,
        index: BlobIndex,
    ) -> Result<Self>
    where
        State: TryFrom<sdk::StateDigest, Error = Error> + std::fmt::Debug + Digestable + Serialize,
    {
        let node_url = env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
        let client = ApiHttpClient {
            url: Url::parse(node_url.as_str()).unwrap(),
            reqwest_client: Client::new(),
        };

        let initial_state = fetch_current_state(&client, &contract_name).await?;

        info!("Fetched current state: {:?}", initial_state);

        let contract_input = ContractInput::<State> {
            initial_state,
            identity,
            tx_hash: "".into(),
            private_blob,
            blobs,
            index,
        };
        let contract_input = bonsai_runner::as_input_data(&contract_input)?;

        Ok(Self {
            client,
            contract_name,
            contract_input,
        })
    }

    pub fn execute(&self) -> Result<()> {
        info!("Checking transition for {}...", self.contract_name);

        let binary = get_binary(self.contract_name.clone())?;
        let execute_info = execute(binary, &self.contract_input)?;
        let output = execute_info.journal.decode::<HyleOutput>().unwrap();
        if !output.success {
            let program_error = std::str::from_utf8(&output.program_outputs).unwrap();
            bail!(
                "\x1b[91mExecution failed ! Program output: {}\x1b[0m",
                program_error
            );
        }
        Ok(())
    }

    pub async fn prove(&self) -> Result<ProofData> {
        info!("Proving transition for {}...", self.contract_name);

        let binary = get_binary(self.contract_name.clone())?;
        let explicit = std::env::var("RISC0_PROVER").unwrap_or_default();
        let receipt = match explicit.to_lowercase().as_str() {
            "bonsai" => bonsai_runner::run_bonsai(binary, self.contract_input.clone()).await?,
            _ => {
                let env = risc0_zkvm::ExecutorEnv::builder()
                    .write_slice(&self.contract_input)
                    .build()
                    .unwrap();

                let prover = risc0_zkvm::default_prover();
                let prove_info = prover.prove(env, binary)?;
                prove_info.receipt
            }
        };

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

        let encoded_receipt = borsh::to_vec(&receipt).expect("Unable to encode receipt");
        Ok(ProofData::Bytes(encoded_receipt))
    }

    pub async fn broadcast_proof(
        &self,
        blob_tx_hash: TxHash,
        proof: ProofData,
    ) -> Result<(), Error> {
        send_proof(
            &self.client,
            blob_tx_hash,
            self.contract_name.clone(),
            proof,
        )
        .await
    }
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

fn execute(binary: &'static [u8], contract_input: &[u8]) -> Result<risc0_zkvm::SessionInfo> {
    let env = risc0_zkvm::ExecutorEnv::builder()
        .write_slice(contract_input)
        .build()
        .unwrap();

    let prover = risc0_zkvm::default_executor();
    Ok(prover.execute(env, binary).unwrap())
}

async fn send_proof(
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
