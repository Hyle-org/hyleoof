use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::{bail, Result};
use client_sdk::{
    rest_client::{IndexerApiHttpClient, NodeApiHttpClient},
    transaction_builder::{ProvableBlobTx, TxExecutorBuilder},
};
use hyllar::{client::metadata::HYLLAR_ELF, erc20::ERC20, Hyllar};
use risc0_zkvm::compute_image_id;
use sdk::{api::APIRegisterContract, BlobTransaction, ContractName, HyleContract, ProgramId};
use tokio::time::timeout;
use tracing::{debug, info};

use crate::{
    app::{HyleOofCtx, States},
    task_manager::Prover,
};

pub async fn init_node(
    node: Arc<NodeApiHttpClient>,
    indexer: Arc<IndexerApiHttpClient>,
) -> Result<()> {
    init_mmid(&node, &indexer).await?;
    init_amm(&node, &indexer).await?;
    init_hyllar2(&node, &indexer).await?;
    init_hyllar(node, indexer).await?;
    Ok(())
}

async fn init_amm(node: &NodeApiHttpClient, indexer: &IndexerApiHttpClient) -> Result<()> {
    match indexer.get_indexer_contract(&"amm".into()).await {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(amm::client::metadata::AMM_ELF)?);
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
            let image_id = hex::encode(compute_image_id(amm::client::metadata::AMM_ELF)?);
            node.register_contract(&APIRegisterContract {
                verifier: "risc0-1".into(),
                program_id: ProgramId(hex::decode(image_id)?),
                state_commitment: amm::Amm::new(BTreeMap::from([(
                    amm::UnorderedTokenPair::new("hyllar".to_string(), "hyllar2".to_string()),
                    (1_000_000_000, 1_000_000_000),
                )]))
                .commit(),
                contract_name: "amm".into(),
            })
            .await?;
            wait_contract_state(indexer, &"amm".into()).await?;
        }
    };

    Ok(())
}

async fn init_hyllar(
    node: Arc<NodeApiHttpClient>,
    indexer: Arc<IndexerApiHttpClient>,
) -> Result<()> {
    match indexer.get_indexer_contract(&"hyllar".into()).await {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(HYLLAR_ELF)?);
            let program_id = hex::encode(contract.program_id.as_slice());
            if program_id != image_id {
                bail!(
                "Invalid Hyllar contract image_id. On-chain version is {program_id}, expected {image_id}",
            );
            }
        }
        Err(e) => {
            bail!("Error fetching Hyllar contract: {e}");
        }
    }

    match indexer
        .fetch_current_state::<Hyllar>(&"hyllar".into())
        .await
    {
        Ok(contract) => {
            if contract.balance_of("amm").is_err() {
                info!("🚀 Initializing Hyllar contract state");

                let executor = TxExecutorBuilder::new(States {
                    hyllar: contract.clone(),
                    hyllar2: indexer.fetch_current_state(&"hyllar2".into()).await?,
                    hydentity: indexer.fetch_current_state(&"hydentity".into()).await?,
                })
                .build();
                let mut app = HyleOofCtx {
                    executor,
                    client: node.clone(),
                    prover: Arc::new(Prover::new(node.clone())),
                    hydentity_cn: "hydentity".into(),
                };
                let mut transaction = ProvableBlobTx::new("faucet.hydentity".into());

                app.verify_hydentity(&mut transaction, "password".into())?;
                app.transfer(
                    &mut transaction,
                    "hyllar".into(),
                    "amm".into(),
                    1_000_000_000,
                )?;
                app.transfer(
                    &mut transaction,
                    "hyllar2".into(),
                    "amm".into(),
                    1_000_000_000,
                )?;
                app.approve(
                    &mut transaction,
                    "hyllar".into(),
                    "amm".into(),
                    1_000_000_000_000_000,
                )?;
                app.approve(
                    &mut transaction,
                    "hyllar2".into(),
                    "amm".into(),
                    1_000_000_000_000_000,
                )?;

                let blob_tx =
                    BlobTransaction::new(transaction.identity.clone(), transaction.blobs.clone());

                let proof_tx_builder = app.executor.process(transaction)?;

                let tx_hash = node.send_tx_blob(&blob_tx).await?;

                info!("🚀 Proving blobs for {tx_hash}");

                for proof in proof_tx_builder.iter_prove() {
                    let proof = proof.await.unwrap();
                    node.send_tx_proof(&proof).await.unwrap();
                }

                timeout(Duration::from_secs(30), async {
                    loop {
                        if let Ok(contract) =indexer.fetch_current_state::<Hyllar>(&"hyllar".into())
                            .await
                        {
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

            let hyllar_token = hyllar::Hyllar::default();

            node.register_contract(&APIRegisterContract {
                verifier: "risc0-1".into(),
                program_id: ProgramId(hex::decode(image_id)?),
                state_commitment: hyllar_token.commit(),
                contract_name: "hyllar2".into(),
            })
            .await?;
            wait_contract_state(indexer, &"hyllar2".into()).await?;
        }
    };

    Ok(())
}

async fn init_mmid(node: &NodeApiHttpClient, indexer: &IndexerApiHttpClient) -> Result<()> {
    match indexer.get_indexer_contract(&"mmid".into()).await {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(hyle_metamask::client::metadata::ELF)?);
            let program_id = hex::encode(contract.program_id.as_slice());
            if program_id != image_id {
                bail!(
                    "Invalid Metamask contract image_id. On-chain version is {program_id}, expected {image_id}",
                );
            }
            info!("✅ Metamask contract is up to date");
        }
        Err(_) => {
            info!("🚀 Registering Metamask contract");
            let image_id = hex::encode(compute_image_id(hyle_metamask::client::metadata::ELF)?);
            node.register_contract(&APIRegisterContract {
                verifier: "risc0-1".into(),
                program_id: ProgramId(hex::decode(image_id)?),
                state_commitment: hyle_metamask::IdentityContractState::new().commit(),
                contract_name: "mmid".into(),
            })
            .await?;
            wait_contract_state(indexer, &"mmid".into()).await?;
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
