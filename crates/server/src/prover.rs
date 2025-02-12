use std::sync::Arc;

use crate::app::AppModuleCtx;
use anyhow::Result;
use client_sdk::helpers::risc0::Risc0Prover;
use hyle::{
    module_handle_messages,
    node_state::module::NodeStateEvent,
    utils::{
        logger::LogMe,
        modules::{module_bus_client, Module},
    },
};
use sdk::{
    BlobTransaction, Block, ContractInput, ContractName, Hashable, HyleOutput, ProofTransaction,
    TransactionData, TxHash,
};
use tracing::{error, info, warn};

pub struct ProverModule {
    bus: ProverModuleBusClient,
    ctx: Arc<AppModuleCtx>,
    unsettled_txs: Vec<BlobTransaction>,
}

module_bus_client! {
#[derive(Debug)]
pub struct ProverModuleBusClient {
    receiver(NodeStateEvent),
}
}

impl Module for ProverModule {
    type Context = Arc<AppModuleCtx>;

    async fn build(ctx: Self::Context) -> Result<Self> {
        let bus = ProverModuleBusClient::new_from_bus(ctx.common.bus.new_handle()).await;

        Ok(ProverModule {
            bus,
            ctx,
            unsettled_txs: vec![],
        })
    }

    async fn run(&mut self) -> Result<()> {
        module_handle_messages! {
            on_bus self.bus,
            listen<NodeStateEvent> event => {
                _ = self.handle_node_state_event(event)
                    .await
                    .log_error("Handling node state event")
            }

        };

        Ok(())
    }
}

impl ProverModule {
    async fn handle_node_state_event(&mut self, event: NodeStateEvent) -> Result<()> {
        let NodeStateEvent::NewBlock(block) = event;
        self.handle_processed_block(*block).await?;

        Ok(())
    }
    async fn handle_processed_block(&mut self, block: Block) -> Result<()> {
        let mut should_trigger = self.unsettled_txs.is_empty();

        for tx in block.txs {
            if let TransactionData::Blob(tx) = tx.transaction_data {
                self.handle_blob(tx);
            }
        }

        for s_tx in block.successful_txs {
            should_trigger = self.settle_tx(s_tx)? == 0 || should_trigger;
        }

        for timedout in block.timed_out_txs {
            should_trigger = self.settle_tx(timedout)? == 0 || should_trigger;
        }

        for failed in block.failed_txs {
            should_trigger = self.settle_tx(failed)? == 0 || should_trigger;
        }

        if should_trigger {
            self.trigger_prove_first();
        }

        Ok(())
    }

    fn handle_blob(&mut self, tx: BlobTransaction) {
        self.unsettled_txs.push(tx);
    }

    fn settle_tx(&mut self, tx: TxHash) -> Result<usize> {
        let tx = self.unsettled_txs.iter().position(|t| t.hash() == tx);
        if let Some(pos) = tx {
            self.unsettled_txs.remove(pos);
            Ok(pos)
        } else {
            Ok(0)
        }
    }

    fn trigger_prove_first(&self) {
        if let Some(tx) = self.unsettled_txs.first().cloned() {
            info!("Triggering prove for tx: {}", tx.hash());
            let ctx = self.ctx.clone();
            tokio::task::spawn(async move {
                match prove_blob_tx(ctx, tx).await {
                    Ok(_) => {}
                    Err(e) => {
                        info!("Error proving tx: {:?}", e);
                    }
                }
            });
        }
    }
}

fn get_prover(cn: &ContractName) -> Option<Risc0Prover> {
    match cn.0.as_str() {
        "hyllar" => Some(Risc0Prover::new(hyllar::client::metadata::HYLLAR_ELF)),
        "hyllar2" => Some(Risc0Prover::new(hyllar::client::metadata::HYLLAR_ELF)),
        "mmid" => Some(Risc0Prover::new(hyle_metamask::client::metadata::ELF)),
        _ => None,
    }
}

async fn prove_blob_tx(ctx: Arc<AppModuleCtx>, tx: BlobTransaction) -> Result<()> {
    let blobs = tx.blobs.clone();
    let tx_hash = tx.hash();
    for (index, blob) in tx.blobs.iter().enumerate() {
        if let Some(prover) = get_prover(&blob.contract_name) {
            info!("Proving tx: {}. Blob for {}", tx_hash, blob.contract_name);
            let contract = ctx.node_client.get_contract(&blob.contract_name).await?;

            let inputs = ContractInput {
                initial_state: contract.state,
                identity: tx.identity.clone(),
                tx_hash: tx_hash.clone(),
                private_input: vec![],
                blobs: blobs.clone(),
                index: sdk::BlobIndex(index),
                tx_ctx: None,
            };

            match prover.prove(inputs).await {
                Ok(proof) => {
                    let tx = ProofTransaction {
                        contract_name: blob.contract_name.clone(),
                        proof,
                    };
                    let _ = ctx
                        .node_client
                        .send_tx_proof(&tx)
                        .await
                        .log_error("failed to send proof to node");

                    let receipt = borsh::from_slice::<risc0_zkvm::Receipt>(&tx.proof.0)?;
                    let ho = receipt.journal.decode::<HyleOutput>();
                    if let Ok(ho) = ho {
                        if !ho.success {
                            let program_error = std::str::from_utf8(&ho.program_outputs).unwrap();
                            warn!("Execution failed ! Program output: {}", program_error);

                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    error!("Error proving tx: {:?}", e);
                }
            }
        }
    }
    Ok(())
}
