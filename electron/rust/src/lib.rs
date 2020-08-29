mod board;
mod prelude;

use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use go_game_engine::{GoGameEngine};


#[wasm_bindgen]
pub fn init() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let board = document.get_element_by_id("board").unwrap();

    board::draw_empty(&board)?;

    let go_game_protector = Rc::new(RefCell::new(GoGameEngine::new(19, 6.5)));

    let socket = board::handle_socket(&board)?;
    board::bind_event(go_game_protector.clone(), &board, &socket)?;

    board::refresh_game_info(&go_game_protector.borrow());

    Ok(())
}
