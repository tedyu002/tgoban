use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use actix_web_actors::ws;

use go_game_engine::{Location, GoGameEngine, ChessType, Player, GameStatus};

use tgoban_ws_protocol as protocol;

const BOARD_SIZE: u8 = 19;
const KOMI_DEFAULT: f64 = 6.5;

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

            let original_status = self.go_game.get_status();

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
                    protocol::Action::GetSGF => {
                        ctx.text(serde_json::to_string_pretty(&protocol::Command::Sgf(
                            self.go_game.to_sgf()
                        )).unwrap());
                        return;
                    },
                    protocol::Action::Refresh => {
                        /* Do nothing */
                    },
                };

                if draw_chess { /* Draw Chess */
                    let mut board: Vec<protocol::ChessType> = Vec::new();
                    for x in 0..BOARD_SIZE {
                        for y in 0..BOARD_SIZE {
                            let location = Location {
                                alphabet: x,
                                digit: y,
                            };

                            let mut is_dead = false;
                            if self.go_game.get_status() == GameStatus::Scoring && !self.go_game.is_alive(location) {
                                is_dead = true;
                            }
                            let chess = match self.go_game.get_chess(location) {
                                ChessType::Black => match is_dead {
                                    false => protocol::ChessType::BlackLive,
                                    true => protocol::ChessType::BlackDead,
                                },
                                ChessType::White => match is_dead {
                                    false => protocol::ChessType::WhiteLive,
                                    true => protocol::ChessType::WhiteDead,
                                },
                                ChessType::None => protocol::ChessType::None,
                            };

                            board.push(chess);
                        }
                    }

                    let command = protocol::Command::Set(board);
                    ctx.text(serde_json::to_string_pretty(&command).unwrap());
                }

                if original_status == GameStatus::Scoring || self.go_game.get_status() == GameStatus::Scoring {
                    /* Draw Belong */
                    let mut belong_board: Vec<protocol::Belong> = Vec::new();

                    for alphabet in 0..BOARD_SIZE {
                        for digit in 0..BOARD_SIZE {
                            belong_board.push(match self.go_game.get_belong(Location {
                                alphabet,
                                digit,
                            }) {
                                None => protocol::Belong::None,
                                Some(player) => {
                                    match player {
                                        Player::Black => {
                                            protocol::Belong::Black
                                        },
                                        Player::White => {
                                            protocol::Belong::White
                                        },
                                    }
                                }
                            });
                        }
                    }
                    let command = protocol::Command::SetBelong(belong_board);
                    ctx.text(serde_json::to_string_pretty(&command).unwrap());
                }

                if self.go_game.get_status() == GameStatus::Scoring {
                    /* Set Score info */
                    let command = protocol::Command::SetScoring(self.go_game.get_score());
                    ctx.text(serde_json::to_string_pretty(&command).unwrap());
                } else {
                    let command = protocol::Command::SetScoring((0.0, 0.0));
                    ctx.text(serde_json::to_string_pretty(&command).unwrap());
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
            komi: self.go_game.komi(),
            capture: [self.go_game.get_capture(&Player::Black), self.go_game.get_capture(&Player::White)],
        })).unwrap());
    }
}

async fn new_go_game(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        GoGame {
            go_game: GoGameEngine::new(BOARD_SIZE, KOMI_DEFAULT),
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
