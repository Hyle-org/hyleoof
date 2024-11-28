use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Tab {
    Register,
    Faucet,
    Transfer,
    Swap,
}

pub enum Msg {
    SwitchTab(Tab),
}

pub struct App {
    active_tab: Tab,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            active_tab: Tab::Faucet, // Onglet par défaut
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SwitchTab(tab) => {
                self.active_tab = tab;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tab_content = match self.active_tab {
            Tab::Register => html! { <crate::register::Register /> },
            Tab::Faucet => html! { <crate::faucet::Faucet /> },
            Tab::Transfer => html! { <crate::transfer::Transfer /> },
            Tab::Swap => html! { <crate::swap::Swap /> },
        };

        html! {
            <div class="container">
                <div class="tabs">
                    <button
                        class={if self.active_tab == Tab::Register { "tab active" } else { "tab" }}
                        onclick={ctx.link().callback(|_| Msg::SwitchTab(Tab::Register))}
                    >
                        {"Register"}
                    </button>
                    <button
                        class={if self.active_tab == Tab::Faucet { "tab active" } else { "tab" }}
                        onclick={ctx.link().callback(|_| Msg::SwitchTab(Tab::Faucet))}
                    >
                        {"Faucet"}
                    </button>
                    <button
                        class={if self.active_tab == Tab::Transfer { "tab active" } else { "tab" }}
                        onclick={ctx.link().callback(|_| Msg::SwitchTab(Tab::Transfer))}
                    >
                        {"Transfer"}
                    </button>
                    <button
                        class={if self.active_tab == Tab::Swap { "tab active" } else { "tab" }}
                        onclick={ctx.link().callback(|_| Msg::SwitchTab(Tab::Swap))}
                    >
                        {"Swap"}
                    </button>
                </div>
                <div class="content">
                    {tab_content}
                </div>
            </div>
        }
    }
}
