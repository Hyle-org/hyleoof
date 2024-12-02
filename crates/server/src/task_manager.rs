use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tracing::{error, info};

use crate::transaction::TransactionBuilder;

pub struct Prover {
    sender: mpsc::UnboundedSender<TransactionBuilder>,
}

impl Prover {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<TransactionBuilder>();
        let receiver = Arc::new(Mutex::new(receiver));

        // Thread parallèle pour traiter les transactions
        tokio::spawn(async move {
            while let Some(tx) = receiver.lock().await.recv().await {
                match tx.prove().await {
                    Ok(_) => info!("Transaction proved"),
                    Err(e) => error!("Failed to prove transaction: {e}"),
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

impl Default for Prover {
    fn default() -> Self {
        Self::new()
    }
}
