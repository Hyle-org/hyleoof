use anyhow::Result;
use axum::{
    extract::Json,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use hydentity::{AccountInfo, Hydentity};
use hyle::rest::client::ApiHttpClient;
use hyllar::HyllarToken;
use reqwest::{Client, Url};
use sdk::{
    erc20::ERC20Action,
    identity_provider::{IdentityAction, IdentityVerification},
    BlobData, ContractInput, ContractName, Identity, TxHash,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use utils::AppError;

mod contract;
mod utils;

#[tokio::main]
async fn main() {
    // Créer un middleware CORS
    let cors = CorsLayer::new()
        .allow_origin(Any) // Permet toutes les origines (peut être restreint)
        .allow_methods(vec![Method::GET, Method::POST]) // Permet les méthodes nécessaires
        .allow_headers(Any); // Permet tous les en-têtes

    let app = Router::new()
        .route("/faucet", post(faucet))
        .route("/transfer", post(transfer))
        .route("/register", post(register))
        .layer(cors); // Appliquer le middleware CORS

    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct Config {
    host: String,
    port: u16,
}

// --------------------------------------------------------
//      Faucet
// --------------------------------------------------------

#[derive(Deserialize)]
struct FaucetRequest {
    username: String,
}

async fn faucet(Json(payload): Json<FaucetRequest>) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_transfer(
        "faucet.hydentity".into(),
        "password".into(),
        payload.username,
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
    amount: u128,
}

async fn transfer(Json(payload): Json<TransferRequest>) -> Result<impl IntoResponse, AppError> {
    let tx_hash = do_transfer(
        payload.username.into(),
        payload.password,
        payload.recipient,
        payload.amount,
    )
    .await?;
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

async fn register_identity(username: String, password: String) -> Result<TxHash, AppError> {
    let config = Config {
        host: "localhost".to_string(),
        port: 4321,
    };
    let url = format!("http://{}:{}", config.host, config.port);
    let client = ApiHttpClient {
        url: Url::parse(url.as_str()).unwrap(),
        reqwest_client: Client::new(),
    };
    let password = password.into_bytes().to_vec();
    let identity_cf = IdentityAction::RegisterIdentity {
        account: username.clone(),
    };
    let blobs = vec![(ContractName("hydentity".to_owned()), identity_cf).into()];
    let hydentity_proof = contract::run(
        &client,
        &"hydentity".into(),
        |token: hydentity::Hydentity| -> ContractInput<hydentity::Hydentity> {
            ContractInput::<Hydentity> {
                initial_state: token,
                identity: username.clone().into(),
                tx_hash: "".to_string(),
                private_blob: BlobData(password.clone()),
                blobs: blobs.clone(),
                index: 0,
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
    let info = state
        .get_identity_info(username)
        .map_err(|_| AppError(StatusCode::NOT_FOUND, anyhow::anyhow!("Identity not found")))?;
    let state: AccountInfo = serde_json::from_str(&info).map_err(|_| {
        AppError(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!("Failed to parse identity info"),
        )
    })?;
    Ok(state.nonce)
}

async fn do_transfer(
    identity: Identity,
    password: String,
    recipient: String,
    amount: u128,
) -> Result<TxHash, AppError> {
    let config = Config {
        host: "localhost".to_string(),
        port: 4321,
    };
    let url = format!("http://{}:{}", config.host, config.port);
    let client = ApiHttpClient {
        url: Url::parse(url.as_str()).unwrap(),
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
        (ContractName("hydentity".to_owned()), identity_cf).into(),
        (ContractName("hyllar".to_owned()), hyllar_cf).into(),
    ];

    let hydentity_proof = contract::run(
        &client,
        &"hydentity".into(),
        |token: hydentity::Hydentity| -> ContractInput<hydentity::Hydentity> {
            ContractInput::<Hydentity> {
                initial_state: token,
                identity: identity.clone(),
                tx_hash: "".to_string(),
                private_blob: BlobData(password.clone()),
                blobs: blobs.clone(),
                index: 0,
            }
        },
    )
    .await?;
    let transfer_proof = contract::run(
        &client,
        &"hyllar".into(),
        |token: hyllar::HyllarToken| -> ContractInput<hyllar::HyllarToken> {
            ContractInput::<HyllarToken> {
                initial_state: token,
                identity: identity.clone(),
                tx_hash: "".to_string(),
                private_blob: BlobData(vec![]),
                blobs: blobs.clone(),
                index: 1,
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
    contract::send_proof(&client, tx_hash.clone(), "hyllar".into(), transfer_proof).await?;

    Ok(tx_hash)
}
