use std::env;

use anyhow::Result;
use axum::{
    extract::Json,
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sdk::{ContractName, Identity, TxHash};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use transaction::TransactionBuilder;
use utils::AppError;

mod contract;
mod transaction;
mod utils;

#[tokio::main]
async fn main() {
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
        .layer(cors); // Appliquer le middleware CORS

    let addr = env::var("HYLEOOF_HOST")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()
        .unwrap();
    info!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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

async fn faucet(Json(payload): Json<FaucetRequest>) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_transfer(
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

async fn transfer(Json(payload): Json<TransferRequest>) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_transfer(
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

async fn approve(Json(payload): Json<ApproveRequest>) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_approve(
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

async fn swap(Json(payload): Json<SwapRequest>) -> Result<impl IntoResponse, AppError> {
    let SwapRequest {
        username,
        password,
        token_a,
        token_b,
        amount,
    } = payload;

    let tx_hash = do_swap(username, password, token_a, token_b, amount).await?;
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

async fn register(Json(payload): Json<RegisterRequest>) -> Result<impl IntoResponse, AppError> {
    let RegisterRequest { username, password } = payload;

    let tx_hash = do_register(username, password).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
// --------------------------------------------------------

async fn do_register(username: Identity, password: String) -> Result<TxHash, AppError> {
    let mut transaction = TransactionBuilder::new(username);

    transaction.register_identity(password);

    let tx_hash = transaction.build().await?;

    Ok(tx_hash)
}

async fn do_transfer(
    identity: Identity,
    password: String,
    recipient: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut transaction = TransactionBuilder::new(identity);

    transaction.verify_identity(password).await?;
    transaction.transfer(token, recipient, amount);

    let tx_hash = transaction.build().await?;

    Ok(tx_hash)
}

async fn do_approve(
    identity: Identity,
    password: String,
    spender: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut transaction = TransactionBuilder::new(identity);

    transaction.verify_identity(password).await?;
    transaction.approve(token, spender, amount);

    let tx_hash = transaction.build().await?;

    Ok(tx_hash)
}

async fn do_swap(
    identity: Identity,
    password: String,
    token_a: ContractName,
    token_b: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let mut transaction = TransactionBuilder::new(identity);

    transaction.verify_identity(password).await?;
    transaction.swap(token_a, token_b, amount).await?;

    let tx_hash = transaction.build().await?;

    Ok(tx_hash)
}
