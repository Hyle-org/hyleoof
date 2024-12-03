use std::{collections::BTreeMap, time::Duration};

use anyhow::{bail, Result};
use hyle::{
    indexer::model::ContractDb, model::RegisterContractTransaction, rest::client::ApiHttpClient,
};
use risc0_zkvm::compute_image_id;
use sdk::{erc20::ERC20, ContractName, Digestable, StateDigest};
use tokio::time::timeout;
use tracing::info;

use crate::{
    contract::fetch_current_state,
    transaction::{States, TransactionBuilder, AMM_BIN, HYLLAR_BIN},
};

pub async fn init_node(client: &ApiHttpClient) -> Result<()> {
    init_amm(client).await?;
    init_hyllar2(client).await?;
    init_hyllar(client).await?;
    Ok(())
}

async fn init_amm(client: &ApiHttpClient) -> Result<()> {
    match client
        .get_indexer_contract(&"amm".into())
        .await?
        .json::<ContractDb>()
        .await
    {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(AMM_BIN)?);
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
            let image_id = hex::encode(compute_image_id(AMM_BIN)?);
            client
                .send_tx_register_contract(&RegisterContractTransaction {
                    owner: "amm".into(),
                    verifier: "risc0".into(),
                    program_id: hex::decode(image_id)?,
                    state_digest: amm::AmmState::new(BTreeMap::from([(
                        amm::UnorderedTokenPair::new("hyllar".to_string(), "hyllar2".to_string()),
                        (1_000_000_000, 1_000_000_000),
                    )]))
                    .as_digest(),
                    contract_name: "amm".into(),
                })
                .await?;
            wait_contract_state(client, &"amm".into()).await?;
        }
    };

    Ok(())
}

async fn init_hyllar(client: &ApiHttpClient) -> Result<()> {
    match client
        .get_indexer_contract(&"hyllar".into())
        .await?
        .json::<ContractDb>()
        .await
    {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(HYLLAR_BIN)?);
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
                    hyllar2: fetch_current_state(client, &"hyllar2".into()).await?,
                    hydentity: fetch_current_state(client, &"hydentity".into()).await?,
                    amm: fetch_current_state(client, &"amm".into()).await?,
                };

                let mut transaction = TransactionBuilder::new("faucet.hydentity".into());

                transaction
                    .verify_identity(&states.hydentity, "password".into())
                    .await
                    .map_err(|e| e.1)?;
                transaction.transfer("hyllar".into(), "amm".into(), 1_000_000_000);
                transaction.approve("hyllar".into(), "amm".into(), 1_000_000_000_000_000);
                transaction.build(&mut states, client).await?;
                transaction.prove(client).await?;

                timeout(Duration::from_secs(30), async {
                    loop {
                        if let Ok(contract) = client
                            .get_indexer_contract(&"hyllar".into())
                            .await
                            .unwrap()
                            .json::<ContractDb>()
                            .await
                        {
                            let contract = hyllar::HyllarTokenContract::init(
                                StateDigest(contract.state_digest).try_into().unwrap(),
                                "faucet.hydentity".into(),
                            );
                            if contract.balance_of("amm") != Ok(1_000_000_000) {
                                info!("⏰ Waiting for Hyllar contract state to be ready");
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

async fn init_hyllar2(client: &ApiHttpClient) -> Result<()> {
    match client
        .get_indexer_contract(&"hyllar2".into())
        .await?
        .json::<ContractDb>()
        .await
    {
        Ok(contract) => {
            let image_id = hex::encode(compute_image_id(HYLLAR_BIN)?);
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
            let image_id = hex::encode(compute_image_id(HYLLAR_BIN)?);

            let mut hyllar_token = hyllar::HyllarTokenContract::init(
                hyllar::HyllarToken::new(100_000_000_000, "faucet.hydentity".to_string()),
                "faucet.hydentity".into(),
            );
            hyllar_token.transfer("amm", 1_000_000_000).unwrap();

            hyllar_token.approve("amm", 1_000_000_000_000_000).unwrap(); // faucet qui approve amm pour
                                                                         // déplacer ses fonds
            let hyllar_state = hyllar_token.state();

            client
                .send_tx_register_contract(&RegisterContractTransaction {
                    owner: "amm".into(),
                    verifier: "risc0".into(),
                    program_id: hex::decode(image_id)?,
                    state_digest: hyllar_state.as_digest(),
                    contract_name: "hyllar2".into(),
                })
                .await?;
            wait_contract_state(client, &"hyllar2".into()).await?;
        }
    };

    Ok(())
}

pub async fn wait_contract_state(
    client: &ApiHttpClient,
    contract: &ContractName,
) -> anyhow::Result<()> {
    timeout(Duration::from_secs(30), async {
        loop {
            let resp = client.get_indexer_contract(contract).await;
            if resp.is_err() || resp.unwrap().json::<ContractDb>().await.is_err() {
                info!("⏰ Waiting for contract {contract} state to be ready");
                tokio::time::sleep(Duration::from_millis(500)).await;
            } else {
                return Ok(());
            }
        }
    })
    .await?
}
