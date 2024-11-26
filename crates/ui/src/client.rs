use anyhow::{bail, Result};
use reqwest::Client;
use serde::Serialize;

use crate::utils::get_server_url;

pub struct WalletClient {
    base_url: String,
    http_client: Client,
}
impl Default for WalletClient {
    fn default() -> Self {
        Self::new(get_server_url())
    }
}

impl WalletClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: Client::new(),
        }
    }

    pub async fn faucet(&self, username: String) -> Result<()> {
        #[derive(Serialize)]
        struct FaucetRequest {
            username: String,
        }

        let request = FaucetRequest { username };

        let url = format!("{}/faucet", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            println!("Faucet request successful for user: {}", request.username);
        } else {
            let error_text = response.text().await?;
            eprintln!("Faucet request failed: {}", error_text);
        }

        Ok(())
    }

    pub async fn transfer(
        &self,
        username: String,
        password: String,
        recipient: String,
        amount: u64,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct TransferRequest {
            username: String,
            password: String,
            recipient: String,
            amount: u64,
        }

        let request = TransferRequest {
            username,
            password,
            recipient,
            amount,
        };

        let url = format!("{}/transfer", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Transfer request failed: {}", error_text);
        }

        Ok(())
    }

    pub async fn register(&self, username: String, password: String) -> Result<()> {
        #[derive(Serialize)]
        struct RegisterRequest {
            username: String,
            password: String,
        }

        let request = RegisterRequest { username, password };

        let url = format!("{}/register", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Registration request failed: {}", error_text);
        }

        Ok(())
    }
}
