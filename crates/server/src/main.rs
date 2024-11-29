use std::env;

use amm::{AmmAction, AmmState};
use anyhow::Result;
use axum::{
    extract::Json,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hydentity::{AccountInfo, Hydentity};
use hyle::rest::client::ApiHttpClient;
use hyllar::HyllarToken;
use reqwest::{Client, Url};
use sdk::{
    erc20::ERC20Action,
    identity_provider::{IdentityAction, IdentityVerification},
    BlobData, BlobIndex, ContractInput, ContractName, Identity, TxHash,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utils::AppError;

mod contract;
mod utils;

static HYLLAR_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/hyllar/hyllar.img");
static HYDENTITY_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/hydentity/hydentity.img");
static AMM_BIN: &[u8] = include_bytes!("../../../../hyle/contracts/amm/amm.img");

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
    username: String,
    password: String,
    token_a: String,
    token_b: String,
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

    let tx_hash = do_swap(username.into(), password, token_a, token_b, amount).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
//      Register
// --------------------------------------------------------

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

async fn register(Json(payload): Json<RegisterRequest>) -> Result<impl IntoResponse, AppError> {
    let tx_hash = register_identity(payload.username.clone(), payload.password.clone()).await?;
    Ok(Json(tx_hash))
}

// --------------------------------------------------------
// --------------------------------------------------------

async fn register_identity(username: String, password: String) -> Result<TxHash, AppError> {
    let node_url = env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
    let client = ApiHttpClient {
        url: Url::parse(node_url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    };
    let password = password.into_bytes().to_vec();
    let identity_cf = IdentityAction::RegisterIdentity {
        account: username.clone(),
    };
    let blobs = vec![identity_cf.as_blob(ContractName("hydentity".to_owned()))];
    let hydentity_proof = contract::run(
        &client,
        &"hydentity".into(),
        HYDENTITY_BIN,
        |token: hydentity::Hydentity| -> ContractInput<hydentity::Hydentity> {
            ContractInput::<Hydentity> {
                initial_state: token,
                identity: username.clone().into(),
                tx_hash: "".into(),
                private_blob: BlobData(password.clone()),
                blobs: blobs.clone(),
                index: 0.into(),
            }
        },
    )
    .await?;

    let tx_hash = contract::send_blobs(&client, username.into(), blobs).await?;
    contract::send_proof(
        &client,
        tx_hash.clone(),
        "hydentity".into(),
        hydentity_proof,
    )
    .await?;
    Ok(tx_hash)
}

async fn get_nonce(client: &ApiHttpClient, username: &str) -> Result<u32, AppError> {
    let state: Hydentity = contract::fetch_current_state(client, &"hydentity".into()).await?;
    info!("State fetched: {:?}", state);
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
    client: &ApiHttpClient,
    token_a: String,
    token_b: String,
    amount: u128,
) -> Result<u128, AppError> {
    let state: AmmState = contract::fetch_current_state(client, &"amm".into()).await?;
    info!("State fetched: {:?}", state);
    let attr = state
        .get_paired_amount(token_a, token_b, amount)
        .ok_or_else(|| AppError(StatusCode::NOT_FOUND, anyhow::anyhow!("Key pair not found")))?;
    Ok(attr)
}

async fn do_transfer(
    identity: Identity,
    password: String,
    recipient: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let node_url = env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
    let client = ApiHttpClient {
        url: Url::parse(node_url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    };

    let nonce = get_nonce(&client, &identity.0).await?;
    let password = password.into_bytes().to_vec();

    let identity_cf = IdentityAction::VerifyIdentity {
        account: identity.0.clone(),
        nonce,
        blobs_hash: vec!["".into()], // TODO: hash blob
    };
    let hyllar_cf = ERC20Action::Transfer { recipient, amount };

    let blobs = vec![
        identity_cf.as_blob(ContractName("hydentity".to_owned())),
        hyllar_cf.as_blob(token.clone(), None, None),
    ];

    let hydentity_proof = contract::run(
        &client,
        &"hydentity".into(),
        HYDENTITY_BIN,
        |token: hydentity::Hydentity| -> ContractInput<hydentity::Hydentity> {
            ContractInput::<Hydentity> {
                initial_state: token,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(password.clone()),
                blobs: blobs.clone(),
                index: 0.into(),
            }
        },
    )
    .await?;
    let transfer_proof = contract::run(
        &client,
        &token.clone(),
        HYLLAR_BIN,
        |token: hyllar::HyllarToken| -> ContractInput<hyllar::HyllarToken> {
            ContractInput::<HyllarToken> {
                initial_state: token,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(vec![]),
                blobs: blobs.clone(),
                index: 1.into(),
            }
        },
    )
    .await?;

    let tx_hash = contract::send_blobs(&client, identity, blobs).await?;
    contract::send_proof(
        &client,
        tx_hash.clone(),
        "hydentity".into(),
        hydentity_proof,
    )
    .await?;
    contract::send_proof(&client, tx_hash.clone(), token, transfer_proof).await?;

    Ok(tx_hash)
}

async fn do_approve(
    identity: Identity,
    password: String,
    spender: String,
    token: ContractName,
    amount: u128,
) -> Result<TxHash, AppError> {
    let node_url = env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
    let client = ApiHttpClient {
        url: Url::parse(node_url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    };

    let nonce = get_nonce(&client, &identity.0).await?;
    let password = password.into_bytes().to_vec();

    let identity_cf = IdentityAction::VerifyIdentity {
        account: identity.0.clone(),
        nonce,
        blobs_hash: vec!["".into()], // TODO: hash blob
    };
    let hyllar_cf = ERC20Action::Approve { spender, amount };

    let blobs = vec![
        identity_cf.as_blob(ContractName("hydentity".to_owned())),
        hyllar_cf.as_blob(token.clone(), None, None),
    ];

    let hydentity_proof = contract::run(
        &client,
        &"hydentity".into(),
        HYDENTITY_BIN,
        |state: hydentity::Hydentity| -> ContractInput<hydentity::Hydentity> {
            ContractInput::<Hydentity> {
                initial_state: state,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(password.clone()),
                blobs: blobs.clone(),
                index: 0.into(),
            }
        },
    )
    .await?;
    let transfer_proof = contract::run(
        &client,
        &token.clone(),
        HYLLAR_BIN,
        |state: hyllar::HyllarToken| -> ContractInput<hyllar::HyllarToken> {
            ContractInput::<HyllarToken> {
                initial_state: state,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(vec![]),
                blobs: blobs.clone(),
                index: 1.into(),
            }
        },
    )
    .await?;

    let tx_hash = contract::send_blobs(&client, identity, blobs).await?;
    contract::send_proof(
        &client,
        tx_hash.clone(),
        "hydentity".into(),
        hydentity_proof,
    )
    .await?;
    contract::send_proof(&client, tx_hash.clone(), token, transfer_proof).await?;

    Ok(tx_hash)
}

async fn do_swap(
    identity: Identity,
    password: String,
    token_a: String,
    token_b: String,
    amount: u128,
) -> Result<TxHash, AppError> {
    let node_url = env::var("NODE_URL").unwrap_or_else(|_| "http://localhost:4321".to_string());
    let client = ApiHttpClient {
        url: Url::parse(node_url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    };

    let nonce = get_nonce(&client, &identity.0).await?;
    let password = password.into_bytes().to_vec();

    let amount_b = get_paired_amount(&client, token_a.clone(), token_b.clone(), amount).await?;

    info!("amount_b: {}", amount_b);

    let identity_cf = IdentityAction::VerifyIdentity {
        account: identity.0.clone(),
        nonce,
        blobs_hash: vec!["".into()], // TODO: hash blob
    };
    let amm_cf = AmmAction::Swap {
        from: identity.clone(),
        pair: (token_a.clone(), token_b.clone()),
    };
    let token_a_transfer_cf = ERC20Action::TransferFrom {
        sender: identity.0.clone(),
        recipient: "amm".into(),
        amount,
    };
    let token_b_transfer_cf = ERC20Action::Transfer {
        recipient: identity.0.clone(),
        amount: amount_b,
    };

    let blobs = vec![
        identity_cf.as_blob(ContractName("hydentity".to_owned())),
        amm_cf.as_blob(
            ContractName("amm".to_owned()),
            None,
            Some(vec![BlobIndex(2), BlobIndex(3)]),
        ),
        token_a_transfer_cf.as_blob(ContractName(token_a.clone()), Some(BlobIndex(1)), None),
        token_b_transfer_cf.as_blob(ContractName(token_b.clone()), Some(BlobIndex(1)), None),
    ];

    let swap_proof = contract::run(
        &client,
        &"amm".into(),
        AMM_BIN,
        |state: AmmState| -> ContractInput<AmmState> {
            ContractInput::<AmmState> {
                initial_state: state,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(vec![]),
                blobs: blobs.clone(),
                index: 1.into(),
            }
        },
    )
    .await?;
    let transfer_a_proof = contract::run(
        &client,
        &token_a.clone().into(),
        HYLLAR_BIN,
        |state: hyllar::HyllarToken| -> ContractInput<hyllar::HyllarToken> {
            ContractInput::<HyllarToken> {
                initial_state: state,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(vec![]),
                blobs: blobs.clone(),
                index: 2.into(),
            }
        },
    )
    .await?;
    let transfer_b_proof = contract::run(
        &client,
        &token_b.clone().into(),
        HYLLAR_BIN,
        |state: hyllar::HyllarToken| -> ContractInput<hyllar::HyllarToken> {
            ContractInput::<HyllarToken> {
                initial_state: state,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(vec![]),
                blobs: blobs.clone(),
                index: 3.into(),
            }
        },
    )
    .await?;

    let hydentity_proof = contract::run(
        &client,
        &"hydentity".into(),
        HYDENTITY_BIN,
        |state: hydentity::Hydentity| -> ContractInput<hydentity::Hydentity> {
            ContractInput::<Hydentity> {
                initial_state: state,
                identity: identity.clone(),
                tx_hash: "".into(),
                private_blob: BlobData(password.clone()),
                blobs: blobs.clone(),
                index: 0.into(),
            }
        },
    )
    .await?;

    let tx_hash = contract::send_blobs(&client, identity, blobs).await?;
    contract::send_proof(
        &client,
        tx_hash.clone(),
        "hydentity".into(),
        hydentity_proof,
    )
    .await?;
    contract::send_proof(&client, tx_hash.clone(), "amm".into(), swap_proof).await?;
    contract::send_proof(&client, tx_hash.clone(), token_a.into(), transfer_a_proof).await?;
    contract::send_proof(&client, tx_hash.clone(), token_b.into(), transfer_b_proof).await?;

    Ok(tx_hash)
}
