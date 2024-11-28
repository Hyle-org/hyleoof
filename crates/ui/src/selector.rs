use yew::prelude::*;

pub enum Msg {
    SelectToken(String),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub on_token_change: Callback<String>, // Callback pour informer le parent du changement
}

pub struct TokenSelector {
    selected_token: String,
}

impl Component for TokenSelector {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            selected_token: "hyllar".to_string(), // Token par défaut
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SelectToken(token) => {
                self.selected_token = token.clone();
                ctx.props().on_token_change.emit(token); // Informe le parent
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let selected_token = self.selected_token.clone();
        html! {
            <div class="token-selector">
                <label for="token-select">{ "Select a token:" }</label>
                <select id="token-select"
                    class="token-dropdown"
                    value={selected_token.clone()}
                    onchange={ctx.link().callback(|e: Event| {
                        let input = e.target_unchecked_into::<web_sys::HtmlSelectElement>();
                        Msg::SelectToken(input.value())
                    })}>
                    <option value="hyllar2">{ "Hyllar2" }</option>
                    <option value="hyllar">{ "Hyllar" }</option>
                </select>
                <p>{ format!("Selected token: {}", self.selected_token) }</p>
            </div>
        }
    }
}
