extern crate zxcvbn;

use std::time::Duration;

use hyllar::{HyllarToken, HyllarTokenContract};
use sdk::erc20::ERC20;
use yew::prelude::*;

use crate::{
    client::WalletClient, contracts::spawn_fetch_state, selector::TokenSelector,
    text_input::TextInput,
};

pub enum Msg {
    SetUserName(String),
    //SetPassword(String),
    SetProgress(String),
    ContractStateUpdate(HyllarToken),
    Faucet,
    TokenChanged(String),
}

#[derive(Default)]
pub struct Faucet {
    username: String,
    //password: String,
    progress: String,
    selected_token: String,
    state: Option<HyllarTokenContract>,
}

impl Faucet {
    fn faucet(ctx: &Context<Self>, username: String, token: String) {
        ctx.link()
            .send_message(Msg::SetProgress("Fauceting...".to_string()));
        ctx.link().send_future(async move {
            match WalletClient::default()
                .faucet(username.clone(), token.clone())
                .await
            {
                Ok(_) => Msg::SetProgress(format!(
                    "Faucet successful for user {}, token {}",
                    username, token
                )),
                Err(e) => Msg::SetProgress(format!("Faucet failed: {}", e)),
            }
        });
    }

    fn display_name(&self) -> String {
        format!("{}.hydentity", self.username)
    }
}

impl Component for Faucet {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_cb = ctx.link().callback(Msg::ContractStateUpdate);
        spawn_fetch_state(state_cb);
        Self {
            selected_token: "hyllar".to_string(),
            ..Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetUserName(next_username) => self.username = next_username,
            //Msg::SetPassword(next_password) => self.password = next_password,
            Msg::SetProgress(progress) => self.progress = progress,
            Msg::Faucet => Self::faucet(ctx, self.display_name(), self.selected_token.clone()),
            Msg::ContractStateUpdate(state) => {
                self.state = Some(HyllarTokenContract::init(
                    state,
                    sdk::Identity(self.username.clone()),
                ))
            }
            Msg::TokenChanged(token) => {
                self.selected_token = token;
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let display_state = match &self.state {
            Some(token) => html! {
                <div>
                <p>{"Total supply: "} {token.total_supply().unwrap()}</p>
                <p>{"Balance: "} {token.balance_of(&self.display_name()).map_or_else(|e| e, |b| b.to_string())}</p>
                </div>
            },
            None => html! {
                <span>{"No state fetched yet"}</span>
            },
        };

        html! {
           <div>
                <div>
                    <TokenSelector
                       on_token_change={ctx.link().callback(Msg::TokenChanged)}
                    />
                    <div>
                        {"Username:"}
                    </div>
                    <div>
                        <TextInput suffix={".hydentity"} on_change={ctx.link().callback(Msg::SetUserName)} value={self.username.clone()}  />
                    </div>
                </div>
                <div class="readout">
                    <button onclick={ctx.link().callback(|_| Msg::Faucet)} class="submit-button">
                        {"Faucet 10 "} { self.selected_token.clone() } {" to "} {self.display_name()}
                    </button>
                </div>
                <div class="progress">
                    {self.progress.clone()}
                </div>
                <div class="state">
                    {display_state}
                </div>
            </div>
        }
    }
}
