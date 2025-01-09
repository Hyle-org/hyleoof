use std::sync::Arc;

use client_sdk::transaction_builder::TransactionBuilder;
use hyle::tools::rest_api_client::NodeApiHttpClient;
use sdk::TxHash;
use tokio::sync::{mpsc, Mutex};
use tracing::error;

pub struct Prover {
    sender: mpsc::UnboundedSender<(TransactionBuilder, TxHash)>,
}

impl Prover {
    pub fn new(node_client: Arc<NodeApiHttpClient>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<(TransactionBuilder, TxHash)>();
        let receiver = Arc::new(Mutex::new(receiver));

        // Thread parallèle pour traiter les transactions
        tokio::spawn(async move {
            while let Some((tx, tx_hash)) = receiver.lock().await.recv().await {
                for (proof, contract_name) in tx.iter_prove() {
                    match proof.await {
                        Ok(proof) => {
                            node_client
                                .send_tx_proof(&hyle::model::ProofTransaction {
                                    proof,
                                    contract_name,
                                    tx_hashes: vec![tx_hash.clone()],
                                })
                                .await
                                .unwrap();
                        }
                        Err(e) => {
                            error!("failed to prove transaction for {contract_name}: {e}");
                            continue;
                        }
                    };
                }
            }
        });

        Prover { sender }
    }

    pub async fn add(&self, tx: TransactionBuilder, tx_hash: TxHash) {
        if let Err(e) = self.sender.send((tx, tx_hash)) {
            eprintln!("Failed to add transaction: {}", e);
        }
    }
}
