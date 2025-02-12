use std::sync::Arc;

use crate::task_manager::Prover;
use crate::utils::AppError;
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

use hyllar::HyllarToken;
use sdk::BlobTransaction;
use sdk::{ContractName, TxHash};
use serde::Deserialize;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

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
    type Context = Arc<AppModuleCtx>;

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

    let executor = TxExecutorBuilder::new(States {
        hyllar,
        hyllar2,
        hydentity,
    })
    .build();

    HyleOofCtx {
        executor,
        client: node.clone(),
        prover: Arc::new(Prover::new(node)),
        hydentity_cn: "hydentity".into(),
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
// --------------------------------------------------------

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

contract_states!(
    pub struct States {
        pub hyllar: HyllarToken,
        pub hyllar2: HyllarToken,
        pub hydentity: Hydentity,
    }
);

pub struct HyleOofCtx {
    pub executor: TxExecutor<States>,
    pub client: Arc<NodeApiHttpClient>,
    pub prover: Arc<Prover>,
    pub hydentity_cn: ContractName,
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
}
