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
    SetRecipient(String),
    SetAmount(String),
    SetProgress(String),
    ContractStateUpdate(HyllarToken),
    TokenChanged(String),
    Transfer,
}

#[derive(Default)]
pub struct Transfer {
    username: String,
    password: String,
    selected_token: String,
    recipient: String,
    progress: String,
    amount: String,
    state: Option<HyllarTokenContract>,
}

impl Transfer {
    fn transfer(
        ctx: &Context<Self>,
        name: String,
        password: String,
        recipient: String,
        token: String,
        amount: u64,
    ) {
        ctx.link()
            .send_message(Msg::SetProgress("Transfering...".to_string()));

        ctx.link().send_future(async move {
            match WalletClient::default()
                .transfer(name.clone(), password, recipient, token, amount)
                .await
            {
                Ok(_) => Msg::SetProgress(format!("Transfer successful for user {}", name)),
                Err(e) => Msg::SetProgress(format!("{}", e)),
            }
        });
    }
    fn display_name(user: &str) -> String {
        format!("{}.hydentity", user)
    }
}

impl Component for Transfer {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_cb = ctx.link().callback(Msg::ContractStateUpdate);
        spawn_fetch_state(state_cb);
        Self {
            amount: "0".to_owned(),
            selected_token: "hyllar".to_owned(),
            ..Self::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetUserName(next_username) => self.username = next_username,
            Msg::SetRecipient(next_username) => self.recipient = next_username,
            Msg::SetPassword(next_password) => self.password = next_password,
            Msg::SetAmount(next_amount) => self.amount = next_amount,
            Msg::SetProgress(progress) => self.progress = progress,
            Msg::Transfer => Self::transfer(
                ctx,
                Self::display_name(&self.username),
                self.password.clone(),
                Self::display_name(&self.recipient),
                self.selected_token.clone(),
                self.amount.parse::<u64>().unwrap_or(0),
            ),
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
                    <TokenSelector
                       on_token_change={ctx.link().callback(Msg::TokenChanged)}
                    />

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
                    <div>
                        {"Recipient:"}
                    </div>
                    <div>
                        <TextInput suffix={".hydentity"} on_change={ctx.link().callback(Msg::SetRecipient)} value={self.recipient.clone()}  />
                    </div>
                    <div>
                        {"Amount:"}
                    </div>
                    <div>
                        <TextInput on_change={ctx.link().callback(Msg::SetAmount)} value={self.amount.clone()}  />
                    </div>
                    <div>{display_balance}</div>
                </div>
                <div class="readout">
                    <button onclick={ctx.link().callback(|_| Msg::Transfer)} class="submit-button">
                        {"Transfer "} {self.amount.clone()} {" "} { self.selected_token.clone() } {" from "} {Self::display_name(&self.username)} {" to "} {Self::display_name(&self.recipient)}

                    </button>
                </div>
                <div class="progress">
                    {self.progress.clone()}
                </div>
            </div>
        }
    }
}
