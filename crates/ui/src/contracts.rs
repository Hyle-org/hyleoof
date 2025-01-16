use client_sdk::rest_client::IndexerApiHttpClient;
use hyllar::HyllarToken;
use yew::{
    platform::{spawn_local, time::sleep},
    Callback,
};

use crate::utils::get_node_url;

use super::faucet::TEN_SECS;

pub fn spawn_fetch_state(state_cb: Callback<HyllarToken>) {
    // Spawn a background task that will fetch state and send it to the component.
    spawn_local(async move {
        let url = get_node_url();
        let client = IndexerApiHttpClient::new(url).unwrap();
        loop {
            let state = match client.fetch_current_state(&"hyllar".into()).await {
                Ok(it) => it,
                Err(_) => {
                    sleep(TEN_SECS).await;
                    continue;
                }
            };

            state_cb.emit(state);

            sleep(TEN_SECS).await;
        }
    });
}
