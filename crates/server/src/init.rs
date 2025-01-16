use std::{collections::BTreeMap, time::Duration};

use anyhow::{bail, Result};
use client_sdk::{
    rest_client::{IndexerApiHttpClient, NodeApiHttpClient},
    transaction_builder::{BuildResult, TransactionBuilder},
};
use hyllar::metadata::HYLLAR_ELF;
use risc0_zkvm::compute_image_id;
use sdk::{
    erc20::ERC20, BlobTransaction, ContractName, Digestable, ProgramId, ProofTransaction,
    RegisterContractTransaction, StateDigest,
};
use tokio::time::timeout;
use tracing::{debug, info};

use crate::States;

pub async fn init_node(node: &NodeApiHttpClient, indexer: &IndexerApiHttpClient) -> Result<()> {
    init_amm(node, indexer).await?;
    init_hyllar2(node, indexer).await?;
    init_hyllar(node, indexer).await?;
    Ok(())
}

async fn init_amm(node: &NodeApiHttpClient, indexer: &IndexerApiHttpClient) -> Result<()> {
    match indexer.get_indexer_contract(&"amm".into()).await {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(amm::metadata::AMM_ELF)?);
            let program_id = hex::encode(contract.program_id.as_slice());
            if program_id != image_id {
                bail!(
                    "Invalid AMM contract image_id. On-chain version is {program_id}, expected {image_id}",
                );
            }
            info!("✅ AMM contract is up to date");
        }
        Err(_) => {
            info!("🚀 Registering AMM contract");
            let image_id = hex::encode(compute_image_id(amm::metadata::AMM_ELF)?);
            node.send_tx_register_contract(&RegisterContractTransaction {
                owner: "amm".into(),
                verifier: "risc0".into(),
                program_id: ProgramId(hex::decode(image_id)?),
                state_digest: amm::AmmState::new(BTreeMap::from([(
                    amm::UnorderedTokenPair::new("hyllar".to_string(), "hyllar2".to_string()),
                    (1_000_000_000, 1_000_000_000),
                )]))
                .as_digest(),
                contract_name: "amm".into(),
            })
            .await?;
            wait_contract_state(indexer, &"amm".into()).await?;
        }
    };

    Ok(())
}

async fn init_hyllar(node: &NodeApiHttpClient, indexer: &IndexerApiHttpClient) -> Result<()> {
    match indexer.get_indexer_contract(&"hyllar".into()).await {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(HYLLAR_ELF)?);
            let program_id = hex::encode(contract.program_id.as_slice());
            if program_id != image_id {
                bail!(
                    "Invalid Hyllar contract image_id. On-chain version is {program_id}, expected {image_id}",
                );
            }
            info!("✅ Hyllar contract is up to date");

            let contract = hyllar::HyllarTokenContract::init(
                StateDigest(contract.state_digest).try_into()?,
                "faucet.hydentity".into(),
            );

            if contract.balance_of("amm").is_err() {
                info!("🚀 Initializing Hyllar contract state");

                let mut states = States {
                    hyllar: contract.state().clone(),
                    hyllar2: indexer.fetch_current_state(&"hyllar2".into()).await?,
                    hydentity: indexer.fetch_current_state(&"hydentity".into()).await?,
                    amm: indexer.fetch_current_state(&"amm".into()).await?,
                };

                let mut transaction = TransactionBuilder::new("faucet.hydentity".into());

                states.verify_identity(&mut transaction, "password".into())?;
                states.transfer(
                    &mut transaction,
                    "hyllar".into(),
                    "amm".into(),
                    1_000_000_000,
                )?;
                states.approve(
                    &mut transaction,
                    "hyllar".into(),
                    "amm".into(),
                    1_000_000_000_000_000,
                )?;

                let BuildResult {
                    identity, blobs, ..
                } = transaction.build(&mut states)?;

                let tx_hash = node
                    .send_tx_blob(&BlobTransaction { identity, blobs })
                    .await?;

                info!("🚀 Proving blobs for {tx_hash}");

                for (proof, contract_name) in transaction.iter_prove() {
                    let proof = proof.await.unwrap();
                    node.send_tx_proof(&ProofTransaction {
                        proof,
                        contract_name,
                    })
                    .await
                    .unwrap();
                }

                timeout(Duration::from_secs(30), async {
                    loop {
                        if let Ok(contract) =node.get_contract(&"hyllar".into())
                            .await
                        {
                            let contract = hyllar::HyllarTokenContract::init(
                                contract.state.try_into().unwrap(),
                                "faucet.hydentity".into(),
                            );
                            let balance = contract.balance_of("amm");
                            if balance != Ok(1_000_000_000) {
                                info!("⏰ Waiting for Hyllar contract state to be ready. amm balance is {balance:?}");
                                debug!("state: {contract:#?}");
                                tokio::time::sleep(Duration::from_millis(500)).await;
                            } else {
                                break;
                            }
                        }
                    }
                })
                .await?;
            }
        }
        Err(_) => {
            bail!("Hyllar contract is not registered");
        }
    };

    Ok(())
}

async fn init_hyllar2(node: &NodeApiHttpClient, indexer: &IndexerApiHttpClient) -> Result<()> {
    match indexer.get_indexer_contract(&"hyllar2".into()).await {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(HYLLAR_ELF)?);
            let program_id = hex::encode(contract.program_id.as_slice());
            if program_id != image_id {
                bail!(
                    "Invalid hyllar 2 contract image_id. On-chain version is {program_id}, expected {image_id}",
                );
            }
            info!("✅ Hyllar2 contract is up to date");
        }
        Err(_) => {
            info!("🚀 Registering Hyllar2 contract");
            let image_id = hex::encode(compute_image_id(HYLLAR_ELF)?);

            let mut hyllar_token = hyllar::HyllarTokenContract::init(
                hyllar::HyllarToken::new(100_000_000_000, "faucet.hydentity".to_string()),
                "faucet.hydentity".into(),
            );
            hyllar_token.transfer("amm", 1_000_000_000).unwrap();

            hyllar_token.approve("amm", 1_000_000_000_000_000).unwrap(); // faucet qui approve amm pour
                                                                         // déplacer ses fonds
            let hyllar_state = hyllar_token.state();

            node.send_tx_register_contract(&RegisterContractTransaction {
                owner: "amm".into(),
                verifier: "risc0".into(),
                program_id: ProgramId(hex::decode(image_id)?),
                state_digest: hyllar_state.as_digest(),
                contract_name: "hyllar2".into(),
            })
            .await?;
            wait_contract_state(indexer, &"hyllar2".into()).await?;
        }
    };

    Ok(())
}

pub async fn wait_contract_state(
    indexer: &IndexerApiHttpClient,
    contract: &ContractName,
) -> anyhow::Result<()> {
    timeout(Duration::from_secs(30), async {
        loop {
            let resp = indexer.get_indexer_contract(contract).await;
            if resp.is_err() {
                info!("⏰ Waiting for contract {contract} state to be ready");
                tokio::time::sleep(Duration::from_millis(500)).await;
            } else {
                return Ok(());
            }
        }
    })
    .await?
}
