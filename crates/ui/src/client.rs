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

    pub async fn faucet(&self, username: String, token: String) -> Result<()> {
        #[derive(Serialize)]
        struct FaucetRequest {
            username: String,
            token: String,
        }

        let request = FaucetRequest { username, token };

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
        token: String,
        amount: u64,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct TransferRequest {
            username: String,
            password: String,
            recipient: String,
            token: String,
            amount: u64,
        }

        let request = TransferRequest {
            username,
            password,
            recipient,
            token,
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

    pub async fn approve(
        &self,
        username: String,
        password: String,
        spender: String,
        token: String,
        amount: u128,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct ApproveRequest {
            username: String,
            password: String,
            spender: String,
            token: String,
            amount: u128,
        }

        let request = ApproveRequest {
            username,
            password,
            spender,
            token,
            amount,
        };

        let url = format!("{}/approve", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Approve request failed: {}", error_text);
        }

        Ok(())
    }

    pub async fn swap(
        &self,
        username: String,
        password: String,
        token_a: String,
        token_b: String,
        amount: u64,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct TransferRequest {
            username: String,
            password: String,
            token_a: String,
            token_b: String,
            amount: u64,
        }

        let request = TransferRequest {
            username,
            password,
            token_a,
            token_b,
            amount,
        };

        let url = format!("{}/swap", self.base_url);
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
