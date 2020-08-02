mod board;
mod prelude;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[macro_use]
extern crate serde_derive;


#[wasm_bindgen]
pub fn init() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let board = document.get_element_by_id("board").unwrap();

    board::draw_empty(&board)?;

    let socket = board::handle_socket(&board)?;
    board::bind_event(&board, &socket)?;

    Ok(())
}
