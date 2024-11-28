#![recursion_limit = "256"]

mod text_input;

mod app;
mod client;
mod contracts;
mod faucet;
mod register;
mod selector;
mod swap;
mod transfer;
mod utils;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
