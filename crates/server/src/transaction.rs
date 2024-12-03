use amm::{AmmAction, AmmState};
use anyhow::{bail, Result};
use axum::http::StatusCode;
use hydentity::{AccountInfo, Hydentity};
use hyle::{
    model::{BlobTransaction, ProofData, ProofTransaction},
    rest::client::ApiHttpClient,
};
use hyllar::HyllarToken;
use sdk::{
    erc20::ERC20Action,
    identity_provider::{IdentityAction, IdentityVerification},
    Blob, BlobData, BlobIndex, ContractName, Identity, TxHash,
};
use tracing::info;

use crate::{contract::ContractRunner, utils::AppError};

pub struct Password(BlobData);

pub static HYLLAR_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/hyllar/hyllar.img");
pub static HYDENTITY_BIN: &[u8] =
    include_bytes!("../../../../hyle/contracts/hydentity/hydentity.img");
pub static AMM_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/amm/amm.img");

pub fn get_binary(contract_name: ContractName) -> Result<&'static [u8]> {
    match contract_name.0.as_str() {
        "hyllar" | "hyllar2" => Ok(HYLLAR_BIN),
        "hydentity" => Ok(HYDENTITY_BIN),
        "amm" => Ok(AMM_BIN),
        _ => bail!("contract {} not supported", contract_name),
    }
}

#[derive(Debug, Clone)]
pub struct States {
    pub hyllar: HyllarToken,
    pub hyllar2: HyllarToken,
    pub hydentity: Hydentity,
    pub amm: AmmState,
}

impl States {
    pub fn for_token<'a>(&'a self, token: &ContractName) -> Result<&'a HyllarToken> {
        match token.0.as_str() {
            "hyllar" => Ok(&self.hyllar),
            "hyllar2" => Ok(&self.hyllar2),
            _ => bail!("Invalid token"),
        }
    }

    pub fn update_for_token(&mut self, token: &ContractName, new_state: HyllarToken) -> Result<()> {
        match token.0.as_str() {
            "hyllar" => self.hyllar = new_state,
            "hyllar2" => self.hyllar2 = new_state,
            _ => bail!("Invalid token"),
        }
        Ok(())
    }
}

pub struct TransactionBuilder {
    identity: Identity,
    hydentity_cf: Vec<(IdentityAction, Password, BlobIndex)>,
    hyllar_cf: Vec<(ERC20Action, ContractName, BlobIndex)>,
    amm_cf: Vec<(AmmAction, BlobIndex)>,
    blobs: Vec<Blob>,
    runners: Vec<ContractRunner>,
    tx_hash: Option<TxHash>,
}

impl TransactionBuilder {
    pub fn new(identity: Identity) -> Self {
        Self {
            identity,
            hydentity_cf: vec![],
            hyllar_cf: vec![],
            amm_cf: vec![],
            blobs: vec![],
            runners: vec![],
            tx_hash: None,
        }
    }

    fn add_hydentity_cf(&mut self, action: IdentityAction, password: Password) {
        self.hydentity_cf
            .push((action.clone(), password, BlobIndex(self.blobs.len() as u32)));
        self.blobs.push(action.as_blob("hydentity".into()));
    }
    fn add_hyllar_cf(
        &mut self,
        token: ContractName,
        action: ERC20Action,
        caller: Option<BlobIndex>,
    ) {
        self.hyllar_cf.push((
            action.clone(),
            token.clone(),
            BlobIndex(self.blobs.len() as u32),
        ));
        self.blobs.push(action.as_blob(token, caller, None));
    }
    fn add_amm_cf(&mut self, action: AmmAction, callees: Vec<BlobIndex>) {
        self.amm_cf
            .push((action.clone(), BlobIndex(self.blobs.len() as u32)));
        self.blobs
            .push(action.as_blob("amm".into(), None, Some(callees)));
    }

    pub async fn verify_identity(
        &mut self,
        state: &Hydentity,
        password: String,
    ) -> Result<(), AppError> {
        let nonce = get_nonce(state, &self.identity.0).await?;
        let password = Password(BlobData(password.into_bytes().to_vec()));

        self.add_hydentity_cf(
            IdentityAction::VerifyIdentity {
                account: self.identity.0.clone(),
                nonce,
                blobs_hash: vec!["".into()], // TODO: hash blob
            },
            password,
        );

        Ok(())
    }

    pub fn register_identity(&mut self, password: String) {
        let password = Password(BlobData(password.into_bytes().to_vec()));

        self.add_hydentity_cf(
            IdentityAction::RegisterIdentity {
                account: self.identity.0.clone(),
            },
            password,
        );
    }

    pub fn approve(&mut self, token: ContractName, spender: String, amount: u128) {
        self.add_hyllar_cf(token, ERC20Action::Approve { spender, amount }, None);
    }

    pub fn transfer(&mut self, token: ContractName, recipient: String, amount: u128) {
        self.add_hyllar_cf(token, ERC20Action::Transfer { recipient, amount }, None);
    }

    pub async fn swap(
        &mut self,
        state: &AmmState,
        token_a: ContractName,
        token_b: ContractName,
        amount: u128,
    ) -> Result<(), AppError> {
        let amount_b =
            get_paired_amount(state, token_a.0.clone(), token_b.0.clone(), amount).await?;

        info!("amount_b: {}", amount_b);
        let swap_blob_index = self.blobs.len() as u32;
        self.add_amm_cf(
            AmmAction::Swap {
                from: self.identity.clone(),
                pair: (token_a.0.clone(), token_b.0.clone()),
            },
            vec![
                BlobIndex(swap_blob_index + 1),
                BlobIndex(swap_blob_index + 2),
            ],
        );
        self.add_hyllar_cf(
            token_a,
            ERC20Action::TransferFrom {
                sender: self.identity.0.clone(),
                recipient: "amm".into(),
                amount,
            },
            Some(BlobIndex(swap_blob_index)),
        );
        self.add_hyllar_cf(
            token_b,
            ERC20Action::Transfer {
                recipient: self.identity.0.clone(),
                amount: amount_b,
            },
            Some(BlobIndex(swap_blob_index)),
        );

        Ok(())
    }

    pub async fn build(&mut self, states: &mut States, client: &ApiHttpClient) -> Result<TxHash> {
        let mut new_states = states.clone();
        for id in self.hydentity_cf.iter() {
            let runner = ContractRunner::new(
                "hydentity".into(),
                get_binary("hydentity".into())?,
                self.identity.clone(),
                id.1 .0.clone(),
                self.blobs.clone(),
                id.2.clone(),
                new_states.hydentity.clone(),
            )
            .await?;
            new_states.hydentity = runner.execute()?.try_into()?;
            self.runners.push(runner);
        }

        for cf in self.hyllar_cf.iter() {
            let runner = ContractRunner::new(
                cf.1.clone(),
                get_binary(cf.1.clone())?,
                self.identity.clone(),
                BlobData(vec![]),
                self.blobs.clone(),
                cf.2.clone(),
                new_states.for_token(&cf.1)?.clone(),
            )
            .await?;
            new_states.update_for_token(&cf.1, runner.execute()?.try_into()?)?;
            self.runners.push(runner);
        }

        for cf in self.amm_cf.iter() {
            let runner = ContractRunner::new::<AmmState>(
                "amm".into(),
                get_binary("amm".into())?,
                self.identity.clone(),
                BlobData(vec![]),
                self.blobs.clone(),
                cf.1.clone(),
                new_states.amm.clone(),
            )
            .await?;
            new_states.amm = runner.execute()?.try_into()?;
            self.runners.push(runner);
        }

        *states = new_states;

        let tx_hash = self.broadcast_blobs(client).await?;

        self.tx_hash = Some(tx_hash.clone());

        Ok(tx_hash)
    }

    async fn broadcast_blobs(&self, client: &ApiHttpClient) -> Result<TxHash> {
        let blob_tx_hash = send_blobs(client, self.identity.clone(), self.blobs.clone()).await?;

        Ok(blob_tx_hash)
    }

    pub async fn prove(&self, client: &ApiHttpClient) -> Result<()> {
        let blob_tx_hash = self.tx_hash.clone().unwrap();

        for runner in self.runners.iter() {
            let proof = runner.prove().await?;
            send_proof(
                client,
                blob_tx_hash.clone(),
                runner.contract_name.clone(),
                proof,
            )
            .await?;
        }

        Ok(())
    }
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

async fn get_nonce(state: &Hydentity, username: &str) -> Result<u32, AppError> {
    let info = state
        .get_identity_info(username)
        .map_err(|err| AppError(StatusCode::NOT_FOUND, anyhow::anyhow!(err)))?;
    let state: AccountInfo = serde_json::from_str(&info).map_err(|_| {
        AppError(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!("Failed to parse identity info"),
        )
    })?;
    Ok(state.nonce)
}

async fn get_paired_amount(
    state: &AmmState,
    token_a: String,
    token_b: String,
    amount: u128,
) -> Result<u128, AppError> {
    let attr = state
        .get_paired_amount(token_a, token_b, amount)
        .ok_or_else(|| AppError(StatusCode::NOT_FOUND, anyhow::anyhow!("Key pair not found")))?;
    Ok(attr)
}
