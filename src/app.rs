extern crate zxcvbn;

use std::time::Duration;

use hyllar::{HyllarToken, HyllarTokenContract};
use sdk::erc20::ERC20;
use yew::prelude::*;

use crate::{
    contracts::{self, spawn_fetch_state},
    faucet::faucet,
    text_input::TextInput,
};
use anyhow::Result;
use futures::{FutureExt, StreamExt};

pub const ONE_SEC: Duration = Duration::from_secs(1);
pub const TEN_SECS: Duration = Duration::from_secs(10);

pub enum Msg {
    SetUserName(String),
    SetPassword(String),
    SetProgress(String),
    ContractStateUpdate(HyllarToken),
    Faucet,
}

#[derive(Default)]
pub struct App {
    username: String,
    password: String,
    progress: String,
    state: Option<HyllarTokenContract>,
}

impl App {
    fn faucet(&self, ctx: &Context<Self>, username: &str) {
        ctx.link()
            .send_message(Msg::SetProgress("Fauceting...".to_string()));
        faucet(username);
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_cb = ctx.link().callback(Msg::ContractStateUpdate);
        spawn_fetch_state(state_cb);
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetUserName(next_username) => self.username = next_username,
            Msg::SetPassword(next_password) => self.password = next_password,
            Msg::SetProgress(progress) => self.progress = progress,
            Msg::Faucet => self.faucet(ctx, &self.username),
            Msg::ContractStateUpdate(state) => {
                self.state = Some(HyllarTokenContract::init(
                    state,
                    sdk::Identity(self.username.clone()),
                ))
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let display_state = match &self.state {
            Some(token) => html! {
                <div>
                <p>{"Total supply: "} {token.total_supply().unwrap()}</p>
                <p>{"Balance: "} {token.balance_of(&self.username).map_or_else(|e| e, |b| b.to_string())}</p>
                </div>
            },
            None => html! {
                <span>{"No state fetched yet"}</span>
            },
        };

        html! {
            <main>
                <div class="entry">
                    <div>
                        {"Username:"}
                    </div>
                    <div>
                        <TextInput on_change={ctx.link().callback(Msg::SetUserName)} value={self.username.clone()} />
                    </div>
                    <div>
                        {"Password:"}
                    </div>
                    <div>
                        <TextInput on_change={ctx.link().callback(Msg::SetPassword)} value={self.password.clone()} />
                    </div>
                </div>
                <div class="readout">
                    <button onclick={ctx.link().callback(|_| Msg::Faucet)}>
                        {"Faucet 10 to user"}
                    </button>
                </div>
        <div class="progress">
                {self.progress.clone()}
        </div>
        <div class="state">
         {display_state}
        </div>
            </main>
        }
    }
}
