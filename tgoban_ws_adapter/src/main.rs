use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;

use go_game_engine::{Location, GoGameEngine, ChessType, Player, GameStatus};

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
                let mut draw_chess = true;
                match action {
                    protocol::Action::Play(location) => {
                        let location = Location {
                            alphabet: location.alphabet,
                            digit: location.digit,
                        };
                        match self.go_game.get_status() {
                            GameStatus::Playing => {
                                match self.go_game.make_move(location) {
                                    Ok(_chess_change) => {},
                                    Err(_) => {
                                        draw_chess = false;
                                    },
                                };
                            },
                            GameStatus::Scoring => {
                                self.go_game.toggle(location);
                            },
                        };
                    },
                    protocol::Action::Back => {
                        self.go_game.regret();
                    },
                    protocol::Action::Pass => {
                        self.go_game.pass();
                        draw_chess = false;
                    },
                    protocol::Action::Refresh => {
                        /* Do nothing */
                    },
                };

                if draw_chess { /* Draw Chess */
                    let mut board: Vec<char> = Vec::new();
                    for x in 0..BOARD_SIZE {
                        for y in 0..BOARD_SIZE {
                            let location = Location {
                                alphabet: x,
                                digit: y,
                            };
                            let mut chess = match self.go_game.get_chess(location) {
                                ChessType::Black => 'B',
                                ChessType::White => 'W',
                                ChessType::None => '0',
                            };

                            if self.go_game.get_status() == GameStatus::Scoring && !self.go_game.is_alive(location) {
                                chess = chess.to_lowercase().next().unwrap();
                            }

                            board.push(chess);
                        }
                    }

                    let command = protocol::Command::Set(board);
                    ctx.text(serde_json::to_string_pretty(&command).unwrap());
                }

                if self.go_game.get_status() == GameStatus::Scoring {
                    let mut score:(i32, i32) = (0, 0);

                    { /* Draw Belong */
                        let mut belong_board: Vec<char> = Vec::new();

                        for alphabet in 0..BOARD_SIZE {
                            for digit in 0..BOARD_SIZE {
                                belong_board.push(match self.go_game.get_belong(Location {
                                    alphabet,
                                    digit,
                                }) {
                                    None => ' ',
                                    Some(player) => {
                                        match player {
                                            Player::Black => {
                                                score.0 += 1;
                                                'B'
                                            },
                                            Player::White => {
                                                score.1 += 1;
                                                'W'
                                            },
                                        }
                                    }
                                });
                            }
                        }
                        let command = protocol::Command::SetBelong(belong_board);
                        ctx.text(serde_json::to_string_pretty(&command).unwrap());
                    }
                    { /* Set Score info */
                        score.0 -= self.go_game.get_capture(&Player::White);
                        score.1 -= self.go_game.get_capture(&Player::Black);

                        let command = protocol::Command::SetScoring(score);
                        ctx.text(serde_json::to_string_pretty(&command).unwrap());
                    }
                }
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
            capture: [self.go_game.get_capture(&Player::Black), self.go_game.get_capture(&Player::White)],
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
