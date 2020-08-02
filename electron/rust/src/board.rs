use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, WebSocket, MessageEvent};
use core::convert::From;
use serde::Deserialize;

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
 
    let rect = document.create_element_ns(Some(SVG_NS), "rect")?;

    rect.set_attribute("width", &format!("{}", CANVAS_SIZE))?;
    rect.set_attribute("height", &format!("{}", CANVAS_SIZE))?;
    rect.set_attribute("fill", "#a57402")?;

    canvas.append_child(&rect)?;

    for row in 0..(BOARD_SIZE as i32) {
        let line = document.create_element_ns(Some(SVG_NS), "line")?;

        line.set_attribute("x1", &format!("{}", LINE_START + CHESS_SIZE * row))?;
        line.set_attribute("y1", &format!("{}", LINE_START))?;
        line.set_attribute("x2", &format!("{}", LINE_START + CHESS_SIZE * row))?;
        line.set_attribute("y2", &format!("{}", LINE_END))?;
        line.set_attribute("stroke", "black")?;

        canvas.append_child(&line)?;
    }

    for col in 0..(BOARD_SIZE as i32) {
        let line = document.create_element_ns(Some(SVG_NS), "line")?;

        line.set_attribute("x1", &format!("{}", LINE_START))?;
        line.set_attribute("y1", &format!("{}", LINE_START + CHESS_SIZE * col))?;
        line.set_attribute("x2", &format!("{}", LINE_END))?;
        line.set_attribute("y2", &format!("{}", LINE_START + CHESS_SIZE * col))?;
        line.set_attribute("stroke", "black")?;

        canvas.append_child(&line)?;
    }

    for row in 0..(BOARD_SIZE as i32) {
        let alphabet = ('A' as u8 + row as u8) as char;
        let x = LINE_START - CHESS_SIZE / 2 + CHESS_SIZE / 4 + CHESS_SIZE * row;
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", x))?;
            text.set_attribute("y", &format!("{}", FONT_SIZE))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            canvas.append_child(&text)?;
        }
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", x))?;
            text.set_attribute("y", &format!("{}", CANVAS_SIZE))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            canvas.append_child(&text)?;
        }
    }

    for col in 0..(BOARD_SIZE as i32) {
        let text = document.create_element_ns(Some(SVG_NS), "text")?;
        let alphabet = 19 - col;
        let y = LINE_START + FONT_SIZE / 2 + CHESS_SIZE * col;
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{:02}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", CHESS_SIZE / 4))?;
            text.set_attribute("y", &format!("{}", y))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            canvas.append_child(&text)?;
        }
        {
            let text = document.create_element_ns(Some(SVG_NS), "text")?;
            let text_node = document.create_text_node(&format!("{:02}", alphabet));
            text.append_child(&text_node)?;
            text.set_attribute("x", &format!("{}", LINE_END + CHESS_SIZE - CHESS_SIZE / 2))?;
            text.set_attribute("y", &format!("{}", y))?;
            text.set_attribute("font-size", &format!("{}", FONT_SIZE))?;
            canvas.append_child(&text)?;
        }
    }

    Ok(())
}

fn convert_location(container: (f64, f64), offset: (i32, i32)) -> Option<(u8, u8)> {
    let x = (offset.0 as f64 / (container.0 / (AREA_NUM as f64))) as u8;
    let y = (offset.1 as f64 / (container.1 / (AREA_NUM as f64))) as u8;

    if x == 0 || y == 0 || x > BOARD_SIZE || y > BOARD_SIZE {
        return None;
    } else {
        return Some((BOARD_SIZE - (y as u8), x as u8 - 1));
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag="Command", content="content")]
pub enum Command {
    Set(Vec<char>),
}

pub fn handle_socket(canvas: &Element) -> Result<WebSocket, JsValue> {
    let ws = WebSocket::new("ws://127.0.0.1:8088/ws/")?;

    {
        let mut canvas = canvas.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let document = web_sys::window().unwrap().document().unwrap();
 
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let raw: Vec<u16> = txt.iter().collect();
                let txt = String::from_utf16(&raw).unwrap();
//TODO                web_sys::window().unwrap().alert_with_message(&txt);
    
                let command: Result<Command, _> = serde_json::from_str(&txt);

                if let Command::Set(board) = command.unwrap() {
                    let children = canvas.children();

                    let mut circles: Vec<Element> = Vec::new();
                    for i in 0..children.length() {
                        let child = children.get_with_index(i).unwrap();

                        if child.tag_name() == "circle" {
                            circles.push(child);
                        }
                    }

                    for element in circles.iter() {
                        element.remove();
                    }

                    for x in 0..(BOARD_SIZE as i32) {
                        for y in 0..(BOARD_SIZE as i32) {
                            let chess = board[(x * (BOARD_SIZE as i32) + y) as usize];

                            let color = match chess {
                                'B' => "black",
                                'W' => "white",
                                _ => continue,
                            };

                            let circle = document.create_element_ns(Some(SVG_NS), "circle").unwrap();

                            circle.set_attribute("cx", &format!("{}", CHESS_SIZE + CHESS_SIZE / 2 + y * CHESS_SIZE));
                            circle.set_attribute("cy", &format!("{}", CHESS_SIZE + CHESS_SIZE / 2 + (BOARD_SIZE as i32 - x - 1) * CHESS_SIZE));

                            circle.set_attribute("stroke", &color);
                            circle.set_attribute("fill", &color);

                            circle.set_attribute("r", &format!("{}", CHESS_SIZE * 2 / 5));

                            canvas.append_child(&circle);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    Ok(ws)
}

pub fn bind_event(canvas: &Element, socket: &WebSocket) -> Result<(), JsValue> {
    {
        let canvas_c = canvas.clone();
        let mut socket = socket.clone();

        let closure = Closure::wrap(Box::new(move |mouse_event: web_sys::MouseEvent| {
            let rect = canvas_c.get_bounding_client_rect();

            if let Some(location) = convert_location((rect.width(), rect.height()), (mouse_event.offset_x(), mouse_event.offset_y())) {
                socket.send_with_str(&format!(r#"{{"Action": "Play", "content": {{"x": {}, "y":{} }} }}"#, location.0, location.1));
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}