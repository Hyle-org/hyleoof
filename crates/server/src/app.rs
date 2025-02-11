use std::sync::Arc;

use crate::task_manager::Prover;
use crate::utils::AppError;
use amm::AmmState;
use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use client_sdk::{
    contract_states,
    rest_client::{IndexerApiHttpClient, NodeApiHttpClient},
    transaction_builder::{ProvableBlobTx, TxExecutor, TxExecutorBuilder},
};
use hydentity::Hydentity;
use hyle::{
    model::CommonRunContext,
    module_handle_messages,
    utils::modules::{module_bus_client, Module},
};

use hyle_metamask::IdentityContractState;
use hyllar::HyllarToken;
use sdk::{identity_provider::IdentityAction, BlobTransaction};
use sdk::{ContractName, Identity, TxHash};
use serde::Deserialize;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub const MMID_CN: &str = "mmid";

pub struct AppModule {
    bus: AppModuleBusClient,
}

pub struct AppModuleCtx {
    pub common: Arc<CommonRunContext>,
    pub node_client: Arc<NodeApiHttpClient>,
    pub indexer_client: Arc<IndexerApiHttpClient>,
}

module_bus_client! {
#[derive(Debug)]
pub struct AppModuleBusClient {
}
}

impl Module for AppModule {
    type Context = AppModuleCtx;

    async fn build(ctx: Self::Context) -> Result<Self> {
        let state = RouterCtx {
            app: Arc::new(Mutex::new(
                build_app_context(ctx.indexer_client.clone(), ctx.node_client.clone()).await,
            )),
        };

        // Créer un middleware CORS
        let cors = CorsLayer::new()
            .allow_origin(Any) // Permet toutes les origines (peut être restreint)
            .allow_methods(vec![Method::GET, Method::POST]) // Permet les méthodes nécessaires
            .allow_headers(Any); // Permet tous les en-têtes

        let api = Router::new()
            .route("/_health", get(health))
            .route("/api/faucet", post(faucet))
            .route("/api/transfer", post(transfer))
            .route("/api/register", post(register))
            .route("/api/swap", post(swap))
            .with_state(state)
            .layer(cors); // Appliquer le middleware CORS

        if let Ok(mut guard) = ctx.common.router.lock() {
            if let Some(router) = guard.take() {
                guard.replace(router.merge(api));
            }
        }
        let bus = AppModuleBusClient::new_from_bus(ctx.common.bus.new_handle()).await;

        Ok(AppModule { bus })
    }

    async fn run(&mut self) -> Result<()> {
        module_handle_messages! {
            on_bus self.bus,
        };

        Ok(())
    }
}

#[derive(Clone)]
struct RouterCtx {
    pub app: Arc<Mutex<HyleOofCtx>>,
}

async fn build_app_context(
    indexer: Arc<IndexerApiHttpClient>,
    node: Arc<NodeApiHttpClient>,
) -> HyleOofCtx {
    let hyllar = indexer.fetch_current_state(&"hyllar".into()).await.unwrap();
    let hyllar2 = indexer
        .fetch_current_state(&"hyllar2".into())
        .await
        .unwrap();
    let hydentity = indexer
        .fetch_current_state(&"hydentity".into())
        .await
        .unwrap();
    let mm_identity = indexer.fetch_current_state(&MMID_CN.into()).await.unwrap();
    let amm = indexer.fetch_current_state(&"amm".into()).await.unwrap();

    let executor = TxExecutorBuilder::new(States {
        hyllar,
        hyllar2,
        hydentity,
        mmid: mm_identity,
        amm,
    })
    .build();

    HyleOofCtx {
        executor,
        client: node.clone(),
        prover: Arc::new(Prover::new(node)),
        hydentity_cn: "hydentity".into(),
        amm_cn: "amm".into(),
    }
}

async fn health() -> impl IntoResponse {
    Json("OK")
}

// --------------------------------------------------------
//      Faucet
// --------------------------------------------------------

#[derive(Deserialize)]
struct FaucetRequest {
    account: String,
    token: ContractName,
}

async fn faucet(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<FaucetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_faucet(ctx, payload.account, payload.token, 10).await?;

    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//      Transfer
// --------------------------------------------------------

#[derive(Deserialize)]
struct TransferRequest {
    account: String,
    recipient: String,
    token: ContractName,
    amount: u128,
}

async fn transfer(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<TransferRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_transfer(
        ctx,
        payload.account.into(),
        payload.recipient,
        payload.token,
        payload.amount,
    )
    .await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//   Swap
// --------------------------------------------------------

#[derive(Deserialize)]
struct SwapRequest {
    account: Identity,
    token_a: ContractName,
    token_b: ContractName,
    amount: u128,
}

async fn swap(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<SwapRequest>,
) -> Result<impl IntoResponse, AppError> {
    let SwapRequest {
        account,
        token_a,
        token_b,
        amount,
    } = payload;

    let tx_hash = do_swap(ctx, account, token_a, token_b, amount).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//      Register
// --------------------------------------------------------

#[derive(Deserialize)]
struct RegisterRequest {
    account: Identity,
}

async fn register(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let RegisterRequest { account } = payload;

    let tx_hash = do_register(ctx, account).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
// --------------------------------------------------------

async fn do_register(ctx: RouterCtx, account: Identity) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(account);

    todo!();
    //app.register_identity(&mut transaction)?;

    app.send(transaction).await
}

async fn do_faucet(
    ctx: RouterCtx,
    recipient: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new("faucet.hydentity".into());

    app.verify_hydentity(&mut transaction, "password".into())?;
    app.transfer(&mut transaction, token, recipient, amount)?;

    app.send(transaction).await
}

async fn do_transfer(
    ctx: RouterCtx,
    identity: Identity,
    recipient: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(identity);

    app.verify_identity(&mut transaction)?;
    app.transfer(&mut transaction, token, recipient, amount)?;

    app.send(transaction).await
}

async fn do_swap(
    ctx: RouterCtx,
    identity: Identity,
    token_a: ContractName,
    token_b: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(identity);

    app.verify_identity(&mut transaction)?;
    app.approve(&mut transaction, token_a.clone(), "amm".to_string(), amount)?;
    app.swap(&mut transaction, token_a, token_b, amount)?;

    app.send(transaction).await
}

contract_states!(
    pub struct States {
        pub hyllar: HyllarToken,
        pub hyllar2: HyllarToken,
        pub hydentity: Hydentity,
        pub mmid: IdentityContractState,
        pub amm: AmmState,
    }
);

pub struct HyleOofCtx {
    pub executor: TxExecutor<States>,
    pub client: Arc<NodeApiHttpClient>,
    pub prover: Arc<Prover>,
    pub hydentity_cn: ContractName,
    pub amm_cn: ContractName,
}

impl HyleOofCtx {
    async fn send(&mut self, transaction: ProvableBlobTx) -> Result<TxHash, AppError> {
        let blob_tx = BlobTransaction {
            identity: transaction.identity.clone(),
            blobs: transaction.blobs.clone(),
        };

        let proof_tx_builder = self.executor.process(transaction)?;

        let tx_hash = self.client.send_tx_blob(&blob_tx).await?;

        self.prover.add(proof_tx_builder).await;

        Ok(tx_hash)
    }

    fn register_identity(
        &mut self,
        transaction: &mut ProvableBlobTx,
        password: String,
    ) -> Result<()> {
        todo!()
        //hydentity::client::register_identity(transaction, self.hydentity_cn.clone(), password)
    }

    pub(crate) fn verify_identity(&mut self, transaction: &mut ProvableBlobTx) -> Result<()> {
        let account = transaction.identity.0.clone();
        let state: &IdentityContractState = &self.executor.mmid;

        info!("State: {:?}", state);

        let nonce = state
            .get_nonce(&account)
            .map_err(|_| anyhow::anyhow!("Account not found"))?;

        transaction.add_action(
            MMID_CN.into(),
            IdentityAction::VerifyIdentity { account, nonce },
            None,
            None,
        )?;

        Ok(())
    }

    pub(crate) fn verify_hydentity(
        &mut self,
        transaction: &mut ProvableBlobTx,
        password: String,
    ) -> Result<()> {
        hydentity::client::verify_identity(
            transaction,
            self.hydentity_cn.clone(),
            &self.executor.hydentity,
            password,
        )
    }

    pub(crate) fn transfer(
        &mut self,
        transaction: &mut ProvableBlobTx,
        token: ContractName,
        recipient: String,
        amount: u128,
    ) -> Result<()> {
        hyllar::client::transfer(transaction, token, recipient, amount)
    }

    pub(crate) fn approve(
        &mut self,
        transaction: &mut ProvableBlobTx,
        token: ContractName,
        spender: String,
        amount: u128,
    ) -> Result<()> {
        hyllar::client::approve(transaction, token, spender, amount)
    }

    pub(crate) fn swap(
        &mut self,
        transaction: &mut ProvableBlobTx,
        token_a: ContractName,
        token_b: ContractName,
        amount: u128,
    ) -> Result<()> {
        let amount_b = Self::get_paired_amount(
            &self.executor.amm,
            token_a.0.clone(),
            token_b.0.clone(),
            amount,
        )?;
        amm::client::swap(
            transaction,
            self.amm_cn.clone(),
            (token_a, token_b),
            (amount, amount_b),
        )
    }

    fn get_paired_amount(
        state: &AmmState,
        token_a: String,
        token_b: String,
        amount: u128,
    ) -> Result<u128> {
        let attr = state
            .get_paired_amount(token_a, token_b, amount)
            .ok_or_else(|| anyhow::anyhow!("Key pair not found"))?;
        Ok(attr)
    }
}
