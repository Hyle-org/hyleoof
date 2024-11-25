use reqwest::Client;
use serde::Serialize;

pub struct WalletClient {
    base_url: String,
    http_client: Client,
}
impl Default for WalletClient {
    fn default() -> Self {
        Self::new("http://127.0.0.1:3000".to_string())
    }
}

impl WalletClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: Client::new(),
        }
    }

    pub async fn faucet(&self, username: String) -> Result<(), reqwest::Error> {
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
        username: &str,
        password: &str,
        nonce: u64,
        recipient: &str,
        amount: u64,
    ) -> Result<(), reqwest::Error> {
        #[derive(Serialize)]
        struct TransferRequest {
            username: String,
            password: String,
            nonce: u64,
            recipient: String,
            amount: u64,
        }

        let request = TransferRequest {
            username: username.to_string(),
            password: password.to_string(),
            nonce,
            recipient: recipient.to_string(),
            amount,
        };

        let url = format!("{}/transfer", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            println!(
                "Transfer of {} to {} successful from user: {}",
                amount, recipient, username
            );
        } else {
            let error_text = response.text().await?;
            eprintln!("Transfer request failed: {}", error_text);
        }

        Ok(())
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<(), reqwest::Error> {
        #[derive(Serialize)]
        struct RegisterRequest {
            username: String,
            password: String,
        }

        let request = RegisterRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let url = format!("{}/register", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            println!("Registration successful for user: {}", username);
        } else {
            let error_text = response.text().await?;
            eprintln!("Registration request failed: {}", error_text);
        }

        Ok(())
    }
}
