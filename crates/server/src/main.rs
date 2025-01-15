use std::{env, sync::Arc};

use amm::AmmState;
use anyhow::{bail, Result};
use axum::{
    extract::{Json, State},
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use client_sdk::transaction_builder::{BuildResult, StateUpdater, TransactionBuilder};
use hydentity::Hydentity;
use hyle::{
    model::BlobTransaction,
    tools::rest_api_client::{IndexerApiHttpClient, NodeApiHttpClient},
};
use hyllar::HyllarToken;
use reqwest::{Client, Url};
use sdk::{BlobIndex, ContractName, Digestable, Identity, StateDigest, TxHash};
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
pub struct RouterCtx {
    pub states: Arc<Mutex<States>>,
    pub client: Arc<NodeApiHttpClient>,
    pub prover: Arc<Prover>,
}

async fn fetch_initial_states(indexer: &IndexerApiHttpClient) -> States {
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

    States {
        hyllar,
        hyllar2,
        hydentity,
        amm,
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

    match init::init_node(&node_client, &indexer_client).await {
        Ok(_) => {}
        Err(e) => {
            error!("Error initializing node: {:?}", e);
            return;
        }
    }

    let state = RouterCtx {
        states: Arc::new(Mutex::new(fetch_initial_states(&indexer_client).await)),
        client: node_client.clone(),
        prover: Arc::new(Prover::new(node_client.clone())),
    };

    info!("Fetched states: {:?}", state.states.lock().await);

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
    let mut states = ctx.states.lock_owned().await;
    let mut transaction = TransactionBuilder::new(username);

    states.register_identity(&mut transaction, password)?;

    send(transaction, &mut states, &ctx.client, &ctx.prover).await
}

async fn do_transfer(
    ctx: RouterCtx,
    identity: Identity,
    password: String,
    recipient: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut states = ctx.states.lock_owned().await;
    let mut transaction = TransactionBuilder::new(identity);

    states.verify_identity(&mut transaction, password)?;
    states.transfer(&mut transaction, token, recipient, amount)?;

    send(transaction, &mut states, &ctx.client, &ctx.prover).await
}

async fn do_approve(
    ctx: RouterCtx,
    identity: Identity,
    password: String,
    spender: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut states = ctx.states.lock_owned().await;
    let mut transaction = TransactionBuilder::new(identity);

    states.verify_identity(&mut transaction, password)?;

    states.approve(&mut transaction, token, spender, amount)?;

    send(transaction, &mut states, &ctx.client, &ctx.prover).await
}

async fn do_swap(
    ctx: RouterCtx,
    identity: Identity,
    password: String,
    token_a: ContractName,
    token_b: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut states = ctx.states.lock_owned().await;
    let mut transaction = TransactionBuilder::new(identity);

    states.register_identity(&mut transaction, password)?;

    states
        .swap(&mut transaction, token_a, token_b, amount)
        .await?;

    send(transaction, &mut states, &ctx.client, &ctx.prover).await
}

async fn send(
    mut transaction: TransactionBuilder,
    states: &mut States,
    client: &NodeApiHttpClient,
    prover: &Arc<Prover>,
) -> Result<TxHash, AppError> {
    let BuildResult {
        identity, blobs, ..
    } = transaction.build(states)?;

    let tx_hash = client
        .send_tx_blob(&BlobTransaction { identity, blobs })
        .await?;

    prover.add(transaction).await;

    Ok(tx_hash)
}

#[derive(Debug, Clone)]
pub struct States {
    pub hyllar: HyllarToken,
    pub hyllar2: HyllarToken,
    pub hydentity: Hydentity,
    pub amm: AmmState,
}

impl States {
    fn register_identity(
        &mut self,
        transaction: &mut TransactionBuilder,
        password: String,
    ) -> Result<()> {
        self.hydentity
            .default_builder(transaction)
            .register_identity(password)
    }

    fn verify_identity(
        &mut self,
        transaction: &mut TransactionBuilder,
        password: String,
    ) -> Result<()> {
        self.hydentity
            .default_builder(transaction)
            .verify_identity(&self.hydentity, password)
    }

    fn transfer(
        &mut self,
        transaction: &mut TransactionBuilder,
        token: ContractName,
        recipient: String,
        amount: u128,
    ) -> Result<()> {
        self.token_builder(token, transaction)
            .transfer(recipient, amount)
    }

    fn approve(
        &mut self,
        transaction: &mut TransactionBuilder,
        token: ContractName,
        spender: String,
        amount: u128,
    ) -> Result<()> {
        transaction.init_with(token.clone(), self.get(&token)?);
        transaction.add_action(
            token,
            hyllar::metadata::HYLLAR_ELF,
            client_sdk::helpers::Prover::Risc0Prover,
            sdk::erc20::ERC20Action::Approve { spender, amount },
            None,
            None,
        )?;
        Ok(())
    }

    pub async fn swap(
        &mut self,
        transaction: &mut TransactionBuilder,
        token_a: ContractName,
        token_b: ContractName,
        amount: u128,
    ) -> Result<()> {
        self.amm_builder(transaction)
            .swap(&self.amm, token_a, token_b, amount)
    }

    fn amm_builder<'b>(&self, builder: &'b mut TransactionBuilder) -> AmmBuilder<'b> {
        builder.init_with("amm".into(), self.amm.as_digest());
        builder.init_with("hyllar".into(), self.hyllar.as_digest());
        builder.init_with("hyllar2".into(), self.hyllar2.as_digest());
        AmmBuilder {
            contract_name: "amm".into(),
            builder,
        }
    }

    fn token_builder<'b>(
        &self,
        token: ContractName,
        builder: &'b mut TransactionBuilder,
    ) -> hyllar::client::Builder<'b> {
        match token.0.as_str() {
            "hyllar" => self.hyllar.default_builder(builder),
            "hyllar2" => self.hyllar2_builder(builder),
            _ => panic!("Unknown token"),
        }
    }

    fn hyllar2_builder<'b>(
        &self,
        builder: &'b mut TransactionBuilder,
    ) -> hyllar::client::Builder<'b> {
        builder.init_with("hyllar2".into(), self.hyllar2.as_digest());
        hyllar::client::Builder {
            contract_name: "hyllar2".into(),
            builder,
        }
    }
}

struct AmmBuilder<'b> {
    contract_name: ContractName,
    builder: &'b mut TransactionBuilder,
}

impl AmmBuilder<'_> {
    fn swap(
        &mut self,
        amm: &AmmState,
        token_a: ContractName,
        token_b: ContractName,
        amount: u128,
    ) -> Result<()> {
        let amount_b = Self::get_paired_amount(amm, token_a.0.clone(), token_b.0.clone(), amount)?;

        let blob_index = self.builder.blobs.len();

        self.builder.add_action(
            self.contract_name.clone(),
            amm::metadata::AMM_ELF,
            client_sdk::helpers::Prover::Risc0Prover,
            amm::AmmAction::Swap {
                pair: (token_a.0.clone(), token_b.0.clone()),
                amounts: (amount, amount_b),
            },
            None,
            Some(vec![BlobIndex(blob_index + 1), BlobIndex(blob_index + 2)]),
        )?;
        self.builder.add_action(
            token_a,
            hyllar::metadata::HYLLAR_ELF,
            client_sdk::helpers::Prover::Risc0Prover,
            sdk::erc20::ERC20Action::TransferFrom {
                sender: self.builder.identity.0.clone(),
                recipient: self.contract_name.0.clone(),
                amount,
            },
            Some(BlobIndex(blob_index)),
            None,
        )?;
        self.builder.add_action(
            token_b,
            hyllar::metadata::HYLLAR_ELF,
            client_sdk::helpers::Prover::Risc0Prover,
            sdk::erc20::ERC20Action::Transfer {
                recipient: self.builder.identity.0.clone(),
                amount: amount_b,
            },
            Some(BlobIndex(blob_index)),
            None,
        )?;
        Ok(())
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

impl StateUpdater for States {
    fn update(&mut self, contract_name: &ContractName, new_state: StateDigest) -> Result<()> {
        match contract_name.0.as_str() {
            "hyllar" => self.hyllar = new_state.try_into()?,
            "hyllar2" => self.hyllar2 = new_state.try_into()?,
            "hydentity" => self.hydentity = new_state.try_into()?,
            "amm" => self.amm = new_state.try_into()?,
            _ => bail!("Unknown contract name"),
        }
        Ok(())
    }

    fn get(&self, contract_name: &ContractName) -> Result<StateDigest> {
        Ok(match contract_name.0.as_str() {
            "hyllar" => self.hyllar.as_digest(),
            "hyllar2" => self.hyllar2.as_digest(),
            "hydentity" => self.hydentity.as_digest(),
            "amm" => self.amm.as_digest(),
            _ => bail!("Unknown contract name"),
        })
    }
}
