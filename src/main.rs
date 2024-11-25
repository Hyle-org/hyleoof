#![recursion_limit = "256"]

mod text_input;

mod app;
mod client;
mod contracts;
mod faucet;
mod register;
mod transfer;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
