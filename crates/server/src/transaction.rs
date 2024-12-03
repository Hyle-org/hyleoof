use amm::{AmmAction, AmmState};
use anyhow::Result;
use axum::http::StatusCode;
use hydentity::{AccountInfo, Hydentity};
use hyle::{model::BlobTransaction, rest::client::ApiHttpClient};
use hyllar::HyllarToken;
use reqwest::{Client, Url};
use sdk::{
    erc20::ERC20Action,
    identity_provider::{IdentityAction, IdentityVerification},
    Blob, BlobData, BlobIndex, ContractName, Identity, StateDigest, TxHash,
};
use tracing::info;

use crate::{
    contract::{ContractRunner, States},
    utils::AppError,
};

pub struct Password(BlobData);

pub struct TransactionBuilder {
    identity: Identity,
    hydentity_cf: Vec<(IdentityAction, Password, BlobIndex)>,
    hyllar_cf: Vec<(ERC20Action, ContractName, BlobIndex)>,
    amm_cf: Vec<(AmmAction, BlobIndex)>,
    blobs: Vec<Blob>,
    runners: Vec<ContractRunner>,
    tx_hash: Option<TxHash>,
}

pub struct ExecutionResult {
    pub contract_name: ContractName,
    pub state_digest: StateDigest,
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

    pub async fn prove(&self) -> Result<()> {
        let blob_tx_hash = self.tx_hash.clone().unwrap();

        for runner in self.runners.iter() {
            let proof = runner.prove().await?;
            runner.broadcast_proof(blob_tx_hash.clone(), proof).await?;
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
