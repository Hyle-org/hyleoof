use yew::prelude::*;

use crate::text_input::TextInput;

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

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
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
                        {"Register"} {self.display_name()}
                    </button>
                </div>
                <div class="progress">
                    {self.progress.clone()}
                </div>
            </div>
        }
    }
}
