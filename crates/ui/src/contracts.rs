use anyhow::Error;
use hyllar::HyllarToken;
use sdk::ContractName;
use serde::Deserialize;
use yew::{
    platform::{spawn_local, time::sleep},
    Callback,
};

use crate::utils::get_node_url;

use super::faucet::TEN_SECS;

#[derive(Deserialize, Debug)]
pub struct Contract {
    pub name: ContractName,
    pub program_id: Vec<u8>,
    pub state: sdk::StateDigest,
    pub verifier: String,
}

pub fn spawn_fetch_state(state_cb: Callback<HyllarToken>) {
    // Spawn a background task that will fetch state and send it to the component.
    spawn_local(async move {
        loop {
            let url = get_node_url();
            let resp = match reqwest::get(format!("{}/v1/contract/hyllar", url)).await {
                Ok(it) => it,
                Err(_) => {
                    sleep(TEN_SECS).await;
                    continue;
                }
            };
            let body = resp.text().await.unwrap();

            if let Ok(contract) = serde_json::from_str::<Contract>(&body) {
                state_cb.emit(contract.state.try_into().unwrap());
            }

            sleep(TEN_SECS).await;
        }
    });
}
