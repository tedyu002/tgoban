use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, WebSocket, MessageEvent};

use tgoban_ws_protocol as protocol;

use crate::prelude::*;

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
    canvas.append_child(&board_group);

    let rect = document.create_element_ns(Some(SVG_NS), "rect")?;

    rect.set_attribute("id", "board_area");
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
        return Some((BOARD_SIZE - (y as u8), x as u8 - 1));
    }
}

pub fn handle_socket(canvas: &Element) -> Result<WebSocket, JsValue> {
    let ws = WebSocket::new("ws://127.0.0.1:8088/ws/")?;

    {
        let canvas = canvas.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let document = web_sys::window().unwrap().document().unwrap();

            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let raw: Vec<u16> = txt.iter().collect();
                let txt = String::from_utf16(&raw).unwrap();

                let command_parse: Result<protocol::Command, _> = serde_json::from_str(&txt);

                match command_parse {
                    Ok(command) => {
                        match command {
                            protocol::Command::Set(board) => {
                                for alphabet in 0..BOARD_SIZE {
                                    for digit in 0..BOARD_SIZE  {
                                        let chess = &board[(alphabet as usize) * (BOARD_SIZE as usize) + digit as usize];

                                        let chess_id = format!("chess_{}_{}", alphabet, digit);
                                        let exist_chess = document.get_element_by_id(&chess_id);

                                        let color = match chess {
                                            protocol::ChessType::BlackLive | protocol::ChessType::BlackDead => "black",
                                            protocol::ChessType::WhiteLive | protocol::ChessType::WhiteDead => "white",
                                            protocol::ChessType::None => {
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
                                                circle.set_attribute("id", &chess_id);

                                                let chess_center = to_chess_center(BOARD_SIZE, alphabet, digit);
                                                circle.set_attribute("cx", &chess_center.0.to_string());
                                                circle.set_attribute("cy", &chess_center.1.to_string());
                                                circle.set_attribute("r", &format!("{}", CHESS_SIZE * 2 / 5));
                                                canvas.append_child(&circle);
                                                circle
                                            }
                                        };

                                        circle.set_attribute("stroke", &color);
                                        circle.set_attribute("fill", &color);

                                        let opacity = match chess {
                                            protocol::ChessType::BlackLive | protocol::ChessType::WhiteLive => "1.0",
                                            protocol::ChessType::BlackDead | protocol::ChessType::WhiteDead => "0.5",
                                            _ => continue,
                                        };

                                        circle.set_attribute("opacity", &opacity);
                                    }
                                }
                            },
                            protocol::Command::SetBelong(belong_board) => {
                                for alphabet in 0..BOARD_SIZE {
                                    for digit in 0..BOARD_SIZE {
                                        let belong = &belong_board[(alphabet as usize) * (BOARD_SIZE as usize) + (digit as usize)];

                                        let belong_id = format!("belong_{}_{}", alphabet, digit);
                                        let exist_belong = document.get_element_by_id(&belong_id);

                                        let color = match belong {
                                            protocol::Belong::Black => "black",
                                            protocol::Belong::White => "white",
                                            protocol::Belong::None => {
                                                if let Some(exist_belong) = exist_belong {
                                                    exist_belong.remove();
                                                }
                                                continue;
                                            },
                                        };

                                        let rect = match exist_belong {
                                            Some(exist_belong) => {
                                                exist_belong
                                            },
                                            None => {
                                                let rect = document.create_element_ns(Some(SVG_NS), "rect").unwrap();

                                                rect.set_attribute("id", &belong_id);

                                                let ratio = 2;
                                                let offset = CHESS_SIZE * ratio / 10;
                                                let size = CHESS_SIZE * (ratio * 2) / 10;

                                                let chess_center = to_chess_center(BOARD_SIZE, alphabet, digit);

                                                rect.set_attribute("x", &(chess_center.0 - offset).to_string());
                                                rect.set_attribute("y", &(chess_center.1 - offset).to_string());
                                                rect.set_attribute("width", &size.to_string());
                                                rect.set_attribute("height", &size.to_string());

                                                canvas.append_child(&rect);
                                                rect
                                            }
                                        };

                                        rect.set_attribute("fill", &color);
                                    }
                                }
                            },
                            protocol::Command::SetGameInfo(game_info) => {
                                let komi = document.get_element_by_id("komi").unwrap();
                                let steps = document.get_element_by_id("steps").unwrap();
                                let now_playing = document.get_element_by_id("now_playing").unwrap();
                                let black_capture = document.get_element_by_id("black_capture").unwrap();
                                let white_capture = document.get_element_by_id("white_capture").unwrap();

                                komi.set_inner_html(&game_info.komi.to_string());
                                steps.set_inner_html(&game_info.steps.to_string());
                                now_playing.set_inner_html(match game_info.playing {
                                    'B' => "Black",
                                    'W' => "White",
                                    _ => {return;}
                                });

                                black_capture.set_inner_html(&game_info.capture[0].to_string());
                                white_capture.set_inner_html(&game_info.capture[1].to_string());
                            },
                            protocol::Command::SetScoring((black_score, white_score)) => {
                                let black_score_disp = document.get_element_by_id("black_score").unwrap();
                                let white_score_disp = document.get_element_by_id("white_score").unwrap();

                                black_score_disp.set_inner_html(&black_score.to_string());
                                white_score_disp.set_inner_html(&white_score.to_string());
                            },
                        }
                    }
                    Err(_) => {
                        web_sys::window().unwrap().alert_with_message(&txt);
                    },
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    {
        let socket = ws.clone();
        let onopen_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            socket.send_with_str(&serde_json::to_string_pretty(&protocol::Action::Refresh).unwrap());
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }

    Ok(ws)
}

pub fn bind_event(canvas: &Element, socket: &WebSocket) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let board_area = canvas.query_selector("#board_area").unwrap().unwrap();

    { // Board click
        let canvas_c = canvas.clone();
        let board_area_c = board_area.clone();
        let socket = socket.clone();

        let closure = Closure::wrap(Box::new(move |mouse_event: web_sys::MouseEvent| {
            let canvas_rect = canvas_c.get_bounding_client_rect();
            let board_rect = board_area_c.get_bounding_client_rect();

            if let Some(location) = convert_location((board_rect.width(), board_rect.height()), (mouse_event.offset_x() as f64, mouse_event.offset_y() as f64 - (canvas_rect.height() - board_rect.height()) / 2.0)) {
                match mouse_event.button() {
                    0 => socket.send_with_str(&serde_json::to_string_pretty(
                            &protocol::Action::Play(
                                protocol::Location {
                                    alphabet: location.0,
                                    digit: location.1,
                                }
                            )
                        ).unwrap()),
                    2 => socket.send_with_str(&serde_json::to_string_pretty(&protocol::Action::Back).unwrap()),
                    _ => {Ok(())},
                };
            }
        }) as Box<dyn FnMut(_)>);
        board_area.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    { // Pass Button
        let socket = socket.clone();
        let button = document.get_element_by_id("pass").unwrap().clone();
        let closure = Closure::wrap(Box::new(move |mouse_event: web_sys::MouseEvent| {
            socket.send_with_str(&serde_json::to_string_pretty(&protocol::Action::Pass).unwrap());
        }) as Box<dyn FnMut(_)>);
        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}

fn to_chess_center(board_size: u8, alphabet: u8, digit: u8) -> (i32, i32) {
    (
        (CHESS_SIZE + CHESS_SIZE / 2) as i32 + (digit as i32) * CHESS_SIZE,
        (CHESS_SIZE + CHESS_SIZE / 2) as i32 + ((board_size - alphabet - 1) as i32) * CHESS_SIZE
    )
}
