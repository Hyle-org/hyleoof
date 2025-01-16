use std::sync::Arc;

use client_sdk::{rest_client::NodeApiHttpClient, transaction_builder::TransactionBuilder};
use sdk::ProofTransaction;
use tokio::sync::{mpsc, Mutex};
use tracing::error;

pub struct Prover {
    sender: mpsc::UnboundedSender<TransactionBuilder>,
}

impl Prover {
    pub fn new(node_client: Arc<NodeApiHttpClient>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<TransactionBuilder>();
        let receiver = Arc::new(Mutex::new(receiver));

        // Thread parallèle pour traiter les transactions
        tokio::spawn(async move {
            while let Some(tx) = receiver.lock().await.recv().await {
                for (proof, contract_name) in tx.iter_prove() {
                    match proof.await {
                        Ok(proof) => {
                            node_client
                                .send_tx_proof(&ProofTransaction {
                                    proof,
                                    contract_name,
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

    pub async fn add(&self, tx: TransactionBuilder) {
        if let Err(e) = self.sender.send(tx) {
            eprintln!("Failed to add transaction: {}", e);
        }
    }
}
