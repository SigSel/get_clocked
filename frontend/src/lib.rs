use std::sync::Arc;

use dominator::{append_dom, body};
use wasm_bindgen::prelude::*;

mod app;
mod pages;

#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    dwind::stylesheet();

    wasm_bindgen_futures::spawn_local(async {
        let state = Arc::new(app::AppState::load().await);
        append_dom(&body(), app::render(state));
    });
}
