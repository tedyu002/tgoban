use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, WebSocket, MessageEvent};

use std::rc::Rc;
use std::cell::RefCell;

use tgoban_ws_protocol as protocol;

use crate::prelude::*;

use go_game_engine::{GoGameEngine, ChessType, Location, Player, GameStatus};

const BOARD_SIZE: u8 = 19;
const CHESS_SIZE: i32 = 100;
const AREA_NUM: i32 = BOARD_SIZE as i32 + 2;
const CANVAS_SIZE: i32 = CHESS_SIZE * AREA_NUM;
const LINE_START: i32 = CHESS_SIZE + CHESS_SIZE / 2;
const LINE_END: i32 = CANVAS_SIZE - LINE_START;
const FONT_SIZE: i32 = CHESS_SIZE / 2;

pub fn draw_empty(canvas: &Element) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    let board_group = document.create_element_ns(Some(SVG_NS), "g")?;
    canvas.append_child(&board_group).expect("Append child error");

    let rect = document.create_element_ns(Some(SVG_NS), "rect")?;

    rect.set_attribute("id", "board_area")?;
    rect.set_attribute("width", &format!("{}", CANVAS_SIZE))?;
    rect.set_attribute("height", &format!("{}", CANVAS_SIZE))?;
    rect.set_attribute("fill", "#a57402")?;

    board_group.append_child(&rect)?;

    for row in 0..(BOARD_SIZE as i32) {
        let line = document.create_element_ns(Some(SVG_NS), "line")?;

        line.set_attribute("x1", &format!("{}", LINE_START + CHESS_SIZE * row))?;
        line.set_attribute("y1", &format!("{}", LINE_START))?;
        line.set_attribute("x2", &format!("{}", LINE_START + CHESS_SIZE * row))?;
        line.set_attribute("y2", &format!("{}", LINE_END))?;
        line.set_attribute("stroke", "black")?;

        board_group.append_child(&line)?;
    }

    for col in 0..(BOARD_SIZE as i32) {
        let line = document.create_element_ns(Some(SVG_NS), "line")?;

        line.set_attribute("x1", &format!("{}", LINE_START))?;
        line.set_attribute("y1", &format!("{}", LINE_START + CHESS_SIZE * col))?;
        line.set_attribute("x2", &format!("{}", LINE_END))?;
        line.set_attribute("y2", &format!("{}", LINE_START + CHESS_SIZE * col))?;
        line.set_attribute("stroke", "black")?;

        board_group.append_child(&line)?;
    }

    for row in 0..(BOARD_SIZE as i32) {
        let mut alphabet = ('A' as u8 + row as u8) as char;
        if alphabet as u8 >= 'I' as u8 {
            alphabet = (alphabet as u8 + 1) as char;
        }

        let x = LINE_START - CHESS_SIZE / 2 + CHESS_SIZE / 4 + CHESS_SIZE * row;
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", x))?;
            text.set_attribute("y", &format!("{}", FONT_SIZE))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            board_group.append_child(&text)?;
        }
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", x))?;
            text.set_attribute("y", &format!("{}", CANVAS_SIZE))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            board_group.append_child(&text)?;
        }
    }

    for col in 0..(BOARD_SIZE as i32) {
        let alphabet = 19 - col;
        let y = LINE_START + FONT_SIZE / 2 + CHESS_SIZE * col;
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{:02}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", CHESS_SIZE / 4))?;
            text.set_attribute("y", &format!("{}", y))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            board_group.append_child(&text)?;
        }
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{:02}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", LINE_END + CHESS_SIZE - CHESS_SIZE / 2))?;
            text.set_attribute("y", &format!("{}", y))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            board_group.append_child(&text)?;
        }
    }

    let stars: [u8; 3] = [3, 9, 15];
    for digit in stars.iter() {
        for alphabet in stars.iter() {
            let circle = document.create_element_ns(Some(SVG_NS), "circle")?;

            let chess_center = to_chess_center(BOARD_SIZE, *alphabet, *digit);

            circle.set_attribute("cx", &chess_center.0.to_string())?;
            circle.set_attribute("cy", &chess_center.1.to_string())?;

            circle.set_attribute("stroke", "black")?;
            circle.set_attribute("fill", "black")?;

            circle.set_attribute("r", &format!("{}", CHESS_SIZE / 10))?;

            board_group.append_child(&circle)?;
        }
    }

    Ok(())
}

fn convert_location(container: (f64, f64), offset: (f64, f64)) -> Option<(u8, u8)> {
    let x = (offset.0 / (container.0 / (AREA_NUM as f64))) as u8;
    let y = (offset.1 / (container.1 / (AREA_NUM as f64))) as u8;

    if x == 0 || y == 0 || x > BOARD_SIZE || y > BOARD_SIZE {
        return None;
    } else {
        return Some((x as u8 - 1, BOARD_SIZE - (y as u8)));
    }
}

pub fn handle_socket(_canvas: &Element) -> Result<WebSocket, JsValue> {
    let ws = WebSocket::new("ws://127.0.0.1:8088")?;

    {
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let _document = web_sys::window().unwrap().document().unwrap();

            if let Ok(_txt) = e.data().dyn_into::<js_sys::JsString>() {
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    {
        let socket = ws.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_e: MessageEvent| {
            socket.send_with_str(&serde_json::to_string_pretty(&protocol::Action::Refresh).unwrap()).expect("Send error");
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }

    Ok(ws)
}

pub fn dispatch_click(go_game: &mut GoGameEngine, location: Location) {
    match go_game.get_status() {
        GameStatus::Playing => {
            match go_game.make_move(location) {
                Ok(_) => {
                    draw_board(&go_game);
                    refresh_game_info(&go_game);
                },
                Err(_) => {
                },
            };
        },
        GameStatus::Scoring => {
            go_game.toggle(location);
            draw_board(&go_game);
            draw_belong(&go_game);
            refresh_game_info(&go_game);
        },
    };
}

pub fn draw_board(go_game: &GoGameEngine) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("board").unwrap();

    for alphabet in 0..go_game.size() {
        for digit in 0..go_game.size() {
            let location = Location {
                alphabet,
                digit,
            };

            let chess = go_game.get_chess(location);

            let chess_id = format!("chess_{}_{}", alphabet, digit);
            let exist_chess = document.get_element_by_id(&chess_id);

            let color = match chess {
                ChessType::Black => "black",
                ChessType::White => "white",
                ChessType::None => {
                    if let Some(exist_chess) = exist_chess {
                        exist_chess.remove();
                    }
                    continue;
                },
            };
            let circle = match exist_chess {
                Some(exist_chess) => {
                    exist_chess
                },
                None => {
                    let circle = document.create_element_ns(Some(SVG_NS), "circle").unwrap();
                    circle.set_attribute("id", &chess_id).expect("Set attribute error");

                    let chess_center = to_chess_center(BOARD_SIZE, alphabet, digit);
                    circle.set_attribute("cx", &chess_center.0.to_string()).expect("Set attribute error");
                    circle.set_attribute("cy", &chess_center.1.to_string()).expect("Set attribute error");
                    circle.set_attribute("r", &format!("{}", CHESS_SIZE * 2 / 5)).expect("Set attribute error");
                    canvas.append_child(&circle).expect("Append child error");
                    circle
                }
            };

            circle.set_attribute("stroke", &color).expect("Set attribute error");
            circle.set_attribute("fill", &color).expect("Set attribute error");

            let opacity = match go_game.is_alive(location) {
                true => "1.0",
                false => "0.5",
            };

            circle.set_attribute("opacity", &opacity).expect("Set attribute error");
        }
    }
}

pub fn draw_belong(go_game: &GoGameEngine) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("board").unwrap();

    for alphabet in 0..go_game.size() {
        for digit in 0..go_game.size() {
            let location = Location {
                alphabet,
                digit,
            };

            let belong = go_game.get_belong(location);

            let belong_id = format!("belong_{}_{}", alphabet, digit);
            let exist_belong = document.get_element_by_id(&belong_id);

            let color = match belong {
                Some(player) => {
                    match player {
                        Player::Black => "black",
                        Player::White => "white",
                    }
                },
                None => {
                    if let Some(exist_belong) = exist_belong {
                        exist_belong.remove();
                    }
                    continue;
                }
            };

            let rect = match exist_belong {
                Some(exist_belong) => {
                    exist_belong
                },
                None => {
                    let rect = document.create_element_ns(Some(SVG_NS), "rect").unwrap();

                    rect.set_attribute("id", &belong_id).expect("Set attribute error");

                    let ratio = 2;
                    let offset = CHESS_SIZE * ratio / 10;
                    let size = CHESS_SIZE * (ratio * 2) / 10;

                    let chess_center = to_chess_center(BOARD_SIZE, alphabet, digit);

                    rect.set_attribute("x", &(chess_center.0 - offset).to_string()).expect("Set attribute error");
                    rect.set_attribute("y", &(chess_center.1 - offset).to_string()).expect("Set attribute error");
                    rect.set_attribute("width", &size.to_string()).expect("Set attribute error");
                    rect.set_attribute("height", &size.to_string()).expect("Set attribute error");

                    canvas.append_child(&rect).expect("Append child error");
                    rect
                }
            };

            rect.set_attribute("fill", &color).expect("Set attribute error");
        }
    }
}

pub fn refresh_game_info(go_game: &GoGameEngine) {
    let document = web_sys::window().unwrap().document().unwrap();

    let komi = document.get_element_by_id("komi").unwrap();
    let steps = document.get_element_by_id("steps").unwrap();
    let now_playing = document.get_element_by_id("now_playing").unwrap();
    let black_capture = document.get_element_by_id("black_capture").unwrap();
    let white_capture = document.get_element_by_id("white_capture").unwrap();
    let black_score_disp = document.get_element_by_id("black_score").unwrap();
    let white_score_disp = document.get_element_by_id("white_score").unwrap();

    komi.set_inner_html(&go_game.komi().to_string());
    steps.set_inner_html(&go_game.steps().to_string());
    now_playing.set_inner_html(match go_game.player() {
        Player::Black => "Black",
        Player::White => "White",
    });

    black_capture.set_inner_html(&go_game.get_capture(&Player::Black).to_string());
    white_capture.set_inner_html(&go_game.get_capture(&Player::White).to_string());

    let scores = go_game.get_score();
    black_score_disp.set_inner_html(&scores.0.to_string());
    white_score_disp.set_inner_html(&scores.1.to_string());
}

pub fn display_sgf(go_game: &GoGameEngine) {
    let document = web_sys::window().unwrap().document().unwrap();
    let sgf_area = document.get_element_by_id("sgf").unwrap();

    sgf_area.dyn_ref::<web_sys::HtmlTextAreaElement>().unwrap().set_value(&go_game.to_sgf());
}

pub fn bind_event(go_game_protector: Rc<RefCell<GoGameEngine>>, canvas: &Element, socket: &WebSocket) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let board_area = canvas.query_selector("#board_area").unwrap().unwrap();

    { // Board click
        let canvas_c = canvas.clone();
        let board_area_c = board_area.clone();
        let _socket = socket.clone();
        let go_game_protector = go_game_protector.clone();

        let closure = Closure::wrap(Box::new(move |mouse_event: web_sys::MouseEvent| {
            let mut go_game = go_game_protector.borrow_mut();
            let canvas_rect = canvas_c.get_bounding_client_rect();
            let board_rect = board_area_c.get_bounding_client_rect();

            if let Some(location) = convert_location((board_rect.width(), board_rect.height()), (mouse_event.offset_x() as f64, mouse_event.offset_y() as f64 - (canvas_rect.height() - board_rect.height()) / 2.0)) {
                match mouse_event.button() {
                    0 => {
                        let location = Location {
                            alphabet: location.0,
                            digit: location.1,
                        };
                        dispatch_click(&mut go_game, location);
                    }
                    2 => {
                        go_game.regret();
                        draw_board(&go_game);
                        draw_belong(&go_game);
                        refresh_game_info(&go_game);
                    },
                    _ => {}
                };
            }
        }) as Box<dyn FnMut(_)>);
        board_area.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    { // Pass Button
        let _socket = socket.clone();
        let button = document.get_element_by_id("pass").unwrap().clone();
        let go_game_protector = go_game_protector.clone();

        let closure = Closure::wrap(Box::new(move |_mouse_event: web_sys::MouseEvent| {
            let mut go_game = go_game_protector.borrow_mut();
            go_game.pass();
            draw_board(&go_game);
            draw_belong(&go_game);
            refresh_game_info(&go_game);
        }) as Box<dyn FnMut(_)>);
        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    { /* SGF */
        let _socket = socket.clone();
        let button = document.get_element_by_id("get_sgf").unwrap().clone();
        let go_game_protector = go_game_protector.clone();
        let closure = Closure::wrap(Box::new(move |_mouse_event: web_sys::MouseEvent| {
            let go_game = go_game_protector.borrow();
            display_sgf(&go_game);
        }) as Box<dyn FnMut(_)>);
        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

fn to_chess_center(board_size: u8, alphabet: u8, digit: u8) -> (i32, i32) {
    (
        (CHESS_SIZE + CHESS_SIZE / 2) as i32 + (alphabet as i32) * CHESS_SIZE,
        (CHESS_SIZE + CHESS_SIZE / 2) as i32 + ((board_size - digit - 1) as i32) * CHESS_SIZE
    )
}
