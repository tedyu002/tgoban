mod protocol;

#[macro_use]
extern crate serde_derive;

use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;

use go_game_engine::{Location, GoGameEngine, ChessType};

const BOARD_SIZE: u8 = 19;

struct GoGame {
    go_game: GoGameEngine,    
}

impl Actor for GoGame {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GoGame {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        if let Ok(ws::Message::Text(text)) = msg {
            let action: Result<protocol::Action, _> = serde_json::from_str(&text);
            
            if let Ok(action) = action {
                match action {
                    protocol::Action::Play(location) => {
                        let mut board: Vec<char> = Vec::new();

                        match self.go_game.make_move(Location {
                            x: location.x,
                            y: location.y,
                        }) {
                            Ok(_chess_change) => {},
                            Err(_) => {return;},
                        }

                        for x in 0..BOARD_SIZE {
                            for y in 0..BOARD_SIZE {
                                let chess = match self.go_game.get_chess(Location {
                                    x,
                                    y,
                                }) {
                                    ChessType::Black => 'B',
                                    ChessType::White => 'W',
                                    ChessType::None => '0',
                                };

                                board.push(chess);
                            }
                        }

                        let command = protocol::Command::Set(board);

                        ctx.text(serde_json::to_string_pretty(&command).unwrap());
                    },
                    protocol::Action::Back => {
                    }
                }
            }
            
        } else if let Ok(ws::Message::Ping(msg)) = msg {
            ctx.pong(&msg);
        }
    }
}

async fn new_go_game(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        GoGame {
            go_game: GoGameEngine::new(BOARD_SIZE),
        },
        &req,
        stream);

    resp
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    /*
    let data =
        web::Data::new(
            Arc::new(RwLock::new(GamesStorage {
                games: BTreeMap::new(),
                sequence: 0,
            }))
        );
        */
    HttpServer::new(move || {
        App::new()
            .route("/ws/", web::get().to(new_go_game))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
