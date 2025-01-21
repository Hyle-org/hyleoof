use std::time::Duration;

use client_sdk::rest_client::IndexerApiHttpClient;
use hyllar::HyllarToken;
use yew::{
    platform::{spawn_local, time::sleep},
    Callback,
};

use crate::utils::get_node_url;

pub const FETCH_INTERVAL: Duration = Duration::from_secs(3);

pub fn spawn_fetch_state(state_cb: Callback<HyllarToken>) {
    // Spawn a background task that will fetch state and send it to the component.
    spawn_local(async move {
        let url = get_node_url();
        let client = IndexerApiHttpClient::new(url).unwrap();
        loop {
            let state = match client.fetch_current_state(&"hyllar".into()).await {
                Ok(it) => it,
                Err(_) => {
                    sleep(FETCH_INTERVAL).await;
                    continue;
                }
            };

            state_cb.emit(state);

            sleep(FETCH_INTERVAL).await;
        }
    });
}
