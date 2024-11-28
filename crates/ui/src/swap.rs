use hyllar::{HyllarToken, HyllarTokenContract};
use sdk::erc20::ERC20;
use yew::prelude::*;

use crate::{
    client::WalletClient, contracts::spawn_fetch_state, selector::TokenSelector,
    text_input::TextInput,
};

pub enum Msg {
    SetUserName(String),
    SetPassword(String),
    SetAmount(String),
    SetProgress(String),
    ContractStateUpdate(HyllarToken),
    TokenAChanged(String),
    TokenBChanged(String),
    Swap,
}

#[derive(Default)]
pub struct Swap {
    username: String,
    password: String,
    selected_token_a: String,
    selected_token_b: String,
    progress: String,
    amount: String,
    state: Option<HyllarTokenContract>,
}

impl Swap {
    fn swap(
        ctx: &Context<Self>,
        name: String,
        password: String,
        token_a: String,
        token_b: String,
        amount: u64,
    ) {
        ctx.link()
            .send_message(Msg::SetProgress("swaping...".to_string()));

        ctx.link().send_future(async move {
            match WalletClient::default()
                .swap(name.clone(), password, token_a, token_b, amount)
                .await
            {
                Ok(_) => Msg::SetProgress(format!("swap successful for user {}", name)),
                Err(e) => Msg::SetProgress(format!("{}", e)),
            }
        });
    }
    fn display_name(user: &str) -> String {
        format!("{}.hydentity", user)
    }
}

impl Component for Swap {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_cb = ctx.link().callback(Msg::ContractStateUpdate);
        spawn_fetch_state(state_cb);
        Self {
            amount: "0".to_owned(),
            selected_token_a: "hyllar".to_owned(),
            selected_token_b: "hyllar2".to_owned(),
            ..Self::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetUserName(next_username) => self.username = next_username,
            Msg::SetPassword(next_password) => self.password = next_password,
            Msg::SetAmount(next_amount) => self.amount = next_amount,
            Msg::SetProgress(progress) => self.progress = progress,
            Msg::Swap => Self::swap(
                ctx,
                Self::display_name(&self.username),
                self.password.clone(),
                self.selected_token_a.clone(),
                self.selected_token_b.clone(),
                self.amount.parse::<u64>().unwrap_or(0),
            ),
            Msg::ContractStateUpdate(state) => {
                self.state = Some(HyllarTokenContract::init(
                    state,
                    sdk::Identity(self.username.clone()),
                ))
            }
            Msg::TokenAChanged(token) => {
                self.selected_token_a = token;
            }
            Msg::TokenBChanged(token) => {
                self.selected_token_b = token;
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let display_balance = match &self.state {
            Some(token) => {
                let balance = token
                    .balance_of(&Self::display_name(&self.username))
                    .unwrap_or(0);
                html! {
                    <span>{format!("Balance: {}", balance)}</span>
                }
            }
            None => html! {
                <span></span>
            },
        };

        html! {
            <div>
                <div>

                    <div>
                        {"Username:"}
                    </div>
                    <div>
                        <TextInput suffix={".hydentity"} on_change={ctx.link().callback(Msg::SetUserName)} value={self.username.clone()}  />
                    </div>
                    <div>
                        {"Password:"}
                    </div>
                    <div>
                        <TextInput on_change={ctx.link().callback(Msg::SetPassword)} value={self.password.clone()}  />
                    </div>
                    <TokenSelector
                       on_token_change={ctx.link().callback(Msg::TokenAChanged)}
                    />
                    <TokenSelector
                       on_token_change={ctx.link().callback(Msg::TokenBChanged)}
                    />
                    <div>
                        {"Amount:"}
                    </div>
                    <div>
                        <TextInput on_change={ctx.link().callback(Msg::SetAmount)} value={self.amount.clone()}  />
                    </div>
                    <div>{display_balance}</div>
                </div>
                <div class="readout">
                    <button onclick={ctx.link().callback(|_| Msg::Swap)} class="submit-button">
                        {"swap "} {self.amount.clone()} {" from "} { self.selected_token_a.clone() } {" to "} { self.selected_token_b.clone() }
                    </button>
                </div>
                <div class="progress">
                    {self.progress.clone()}
                </div>
            </div>
        }
    }
}
