use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;

use go_game_engine::{Location, GoGameEngine, ChessType, Player};

use tgoban_ws_protocol as protocol;

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
                        match self.go_game.make_move(Location {
                            alphabet: location.alphabet,
                            digit: location.digit,
                        }) {
                            Ok(_chess_change) => {},
                            Err(_) => {return;},
                        }
                    },
                    protocol::Action::Back => {
                        self.go_game.regret();
                    }
                };

                let mut board: Vec<char> = Vec::new();
                for x in 0..BOARD_SIZE {
                    for y in 0..BOARD_SIZE {
                        let chess = match self.go_game.get_chess(Location {
                            alphabet: x,
                            digit: y,
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
            }
        } else if let Ok(ws::Message::Ping(msg)) = msg {
            ctx.pong(&msg);
        }
        ctx.text(serde_json::to_string_pretty(&protocol::Command::SetGameInfo(protocol::GameInfo {
            steps: self.go_game.steps(),
            playing: match self.go_game.player() {
                Player::Black => 'B',
                Player::White => 'W',
            },
            deads: [self.go_game.deads(&Player::Black), self.go_game.deads(&Player::White)],
        })).unwrap());
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
