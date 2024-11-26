use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = SERVER_URL)]
    static SERVER_URL: String;
    #[wasm_bindgen(js_namespace = window, js_name = NODE_URL)]
    static NODE_URL: String;
}

pub fn get_server_url() -> String {
    SERVER_URL.clone()
}

pub fn get_node_url() -> String {
    NODE_URL.clone()
}
