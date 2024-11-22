pub fn faucet(username: &str) {
    web_sys::console::log_1(&username.into());
}
