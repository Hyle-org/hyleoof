use yew::prelude::*;

use crate::{client::WalletClient, text_input::TextInput};

pub enum Msg {
    SetUserName(String),
    SetPassword(String),
    SetProgress(String),
    Register,
}

#[derive(Default)]
pub struct Register {
    username: String,
    password: String,
    progress: String,
}

impl Register {
    fn register(ctx: &Context<Self>, name: String, password: String) {
        ctx.link()
            .send_message(Msg::SetProgress("Registering...".to_string()));

        ctx.link().send_future(async move {
            match WalletClient::default()
                .register(name.clone(), password)
                .await
            {
                Ok(_) => Msg::SetProgress(format!("Register successful for user {}", name)),
                Err(e) => Msg::SetProgress(format!("Register failed: {}", e)),
            }
        });
    }

    fn display_name(&self) -> String {
        format!("{}.hydentity", self.username)
    }
}

impl Component for Register {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetUserName(next_username) => self.username = next_username,
            Msg::SetPassword(next_password) => self.password = next_password,
            Msg::SetProgress(progress) => self.progress = progress,
            Msg::Register => Self::register(ctx, self.display_name(), self.password.clone()),
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                </div>
                <div class="readout">
                    <button onclick={ctx.link().callback(|_| Msg::Register)} class="submit-button">
                        {"Register "} {self.display_name()}
                    </button>
                </div>
                <div class="progress">
                    {self.progress.clone()}
                </div>
            </div>
        }
    }
}
