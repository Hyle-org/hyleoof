use std::{env, sync::Arc};

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
use hyllar::HyllarToken;
use reqwest::{Client, Url};
use sdk::BlobTransaction;
use sdk::{ContractName, Identity, TxHash};
use serde::Deserialize;
use task_manager::Prover;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utils::AppError;

mod init;
mod task_manager;
mod utils;

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
    let amm = indexer.fetch_current_state(&"amm".into()).await.unwrap();

    let executor = TxExecutorBuilder::new(States {
        hyllar,
        hyllar2,
        hydentity,
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

fn setup_tracing() {
    let mut filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .unwrap();
    let var = std::env::var("RUST_LOG").unwrap_or("".to_string());
    if !var.contains("risc0_zkvm") {
        filter = filter.add_directive("risc0_zkvm=info".parse().unwrap());
        filter = filter.add_directive("risc0_circuit_rv32im=info".parse().unwrap());
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let node_url = env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
    let indexer_url =
        env::var("INDEXER_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
    let node_client = Arc::new(NodeApiHttpClient {
        url: Url::parse(node_url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    });
    let indexer_client = Arc::new(IndexerApiHttpClient {
        url: Url::parse(indexer_url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    });

    match init::init_node(node_client.clone(), indexer_client.clone()).await {
        Ok(_) => {}
        Err(e) => {
            error!("Error initializing node: {:?}", e);
            return;
        }
    }

    let state = RouterCtx {
        app: Arc::new(Mutex::new(
            build_app_context(indexer_client, node_client).await,
        )),
    };

    // Créer un middleware CORS
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permet toutes les origines (peut être restreint)
        .allow_methods(vec![Method::GET, Method::POST]) // Permet les méthodes nécessaires
        .allow_headers(Any); // Permet tous les en-têtes

    let app = Router::new()
        .route("/_health", get(health))
        .route("/api/faucet", post(faucet))
        .route("/api/transfer", post(transfer))
        .route("/api/register", post(register))
        .route("/api/approve", post(approve))
        .route("/api/swap", post(swap))
        .with_state(state)
        .layer(cors); // Appliquer le middleware CORS

    let addr: String = env::var("HYLEOOF_HOST")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()
        .unwrap();
    info!("Server running on {}", addr);
    _ = axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app).await;
}

async fn health() -> impl IntoResponse {
    Json("OK")
}

// --------------------------------------------------------
//      Faucet
// --------------------------------------------------------

#[derive(Deserialize)]
struct FaucetRequest {
    username: String,
    token: ContractName,
}

async fn faucet(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<FaucetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_transfer(
        ctx,
        "faucet.hydentity".into(),
        "password".into(),
        payload.username,
        payload.token,
        10,
    )
    .await?;

    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//      Transfer
// --------------------------------------------------------

#[derive(Deserialize)]
struct TransferRequest {
    username: String,
    password: String,
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
        payload.username.into(),
        payload.password,
        payload.recipient,
        payload.token,
        payload.amount,
    )
    .await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//    Approve
// --------------------------------------------------------

#[derive(Deserialize)]
struct ApproveRequest {
    username: String,
    password: String,
    spender: String,
    token: String,
    amount: u128,
}

async fn approve(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<ApproveRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_approve(
        ctx,
        payload.username.into(),
        payload.password,
        payload.spender,
        payload.token.into(),
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
    username: Identity,
    password: String,
    token_a: ContractName,
    token_b: ContractName,
    amount: u128,
}

async fn swap(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<SwapRequest>,
) -> Result<impl IntoResponse, AppError> {
    let SwapRequest {
        username,
        password,
        token_a,
        token_b,
        amount,
    } = payload;

    let tx_hash = do_swap(ctx, username, password, token_a, token_b, amount).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//      Register
// --------------------------------------------------------

#[derive(Deserialize)]
struct RegisterRequest {
    username: Identity,
    password: String,
}

async fn register(
    State(ctx): State<RouterCtx>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let RegisterRequest { username, password } = payload;

    let tx_hash = do_register(ctx, username, password).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
// --------------------------------------------------------

async fn do_register(
    ctx: RouterCtx,
    username: Identity,
    password: String,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(username);

    app.register_identity(&mut transaction, password)?;

    app.send(transaction).await
}

async fn do_transfer(
    ctx: RouterCtx,
    identity: Identity,
    password: String,
    recipient: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(identity);

    app.verify_identity(&mut transaction, password)?;
    app.transfer(&mut transaction, token, recipient, amount)?;

    app.send(transaction).await
}

async fn do_approve(
    ctx: RouterCtx,
    identity: Identity,
    password: String,
    spender: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(identity);

    app.verify_identity(&mut transaction, password)?;

    app.approve(&mut transaction, token, spender, amount)?;

    app.send(transaction).await
}

async fn do_swap(
    ctx: RouterCtx,
    identity: Identity,
    password: String,
    token_a: ContractName,
    token_b: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut app = ctx.app.lock_owned().await;
    let mut transaction = ProvableBlobTx::new(identity);

    app.verify_identity(&mut transaction, password)?;
    app.swap(&mut transaction, token_a, token_b, amount).await?;

    app.send(transaction).await
}

contract_states!(
    pub struct States {
        pub hyllar: HyllarToken,
        pub hyllar2: HyllarToken,
        pub hydentity: Hydentity,
        pub amm: AmmState,
    }
);

struct HyleOofCtx {
    executor: TxExecutor<States>,
    client: Arc<NodeApiHttpClient>,
    prover: Arc<Prover>,
    hydentity_cn: ContractName,
    amm_cn: ContractName,
}

impl HyleOofCtx {
    async fn send(&mut self, transaction: ProvableBlobTx) -> Result<TxHash, AppError> {
        let blob_tx = BlobTransaction::new(transaction.identity.clone(), transaction.blobs.clone());

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
        hydentity::client::register_identity(transaction, self.hydentity_cn.clone(), password)
    }

    fn verify_identity(
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

    fn transfer(
        &mut self,
        transaction: &mut ProvableBlobTx,
        token: ContractName,
        recipient: String,
        amount: u128,
    ) -> Result<()> {
        hyllar::client::transfer(transaction, token, recipient, amount)
    }

    fn approve(
        &mut self,
        transaction: &mut ProvableBlobTx,
        token: ContractName,
        spender: String,
        amount: u128,
    ) -> Result<()> {
        hyllar::client::approve(transaction, token, spender, amount)
    }

    pub async fn swap(
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
