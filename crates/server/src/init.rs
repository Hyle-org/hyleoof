use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::{bail, Result};
use client_sdk::{
    rest_client::{IndexerApiHttpClient, NodeApiHttpClient},
    transaction_builder::{ProvableBlobTx, TxExecutorBuilder},
};
use hyllar::client::metadata::HYLLAR_ELF;
use risc0_zkvm::compute_image_id;
use sdk::{
    api::APIRegisterContract, erc20::ERC20, BlobTransaction, ContractName, Digestable, ProgramId,
    StateDigest,
};
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
            info!("✅ Hyllar contract is up to date");

            let contract = hyllar::HyllarTokenContract::init(
                StateDigest(contract.state_digest).try_into()?,
                "faucet.hydentity".into(),
            );

            if contract.balance_of("amm").is_err() {
                info!("🚀 Initializing Hyllar contract state");

                let executor = TxExecutorBuilder::new(States {
                    hyllar: contract.state().clone(),
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
                app.approve(
                    &mut transaction,
                    "hyllar".into(),
                    "amm".into(),
                    1_000_000_000_000_000,
                )?;

                let blob_tx = BlobTransaction {
                    identity: transaction.identity.clone(),
                    blobs: transaction.blobs.clone(),
                };

                let proof_tx_builder = app.executor.process(transaction)?;

                let tx_hash = node.send_tx_blob(&blob_tx).await?;

                info!("🚀 Proving blobs for {tx_hash}");

                for proof in proof_tx_builder.iter_prove() {
                    let proof = proof.await.unwrap();
                    node.send_tx_proof(&proof).await.unwrap();
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

            node.register_contract(&APIRegisterContract {
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
