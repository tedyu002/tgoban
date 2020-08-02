mod protocol;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, Arc, RwLock};

use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get, post, Result};
use actix_web_actors::ws;
use serde::Deserialize;

#[macro_use]
extern crate serde_derive;

use go_game_engine::GoGameEngine;

const BOARD_SIZE: u8 = 19;

/*
#[derive(Deserialize)]
struct Location {
    x: u8,
    y: u8,
}

struct GamesStorage {
    games: BTreeMap<u64, Arc<RwLock<GoGameEngine>>>,
    sequence: u64,
}

fn get_game(game_storage_locker: &Arc<RwLock<GamesStorage>>, id: u64) -> Option<Arc<RwLock<GoGameEngine>>> {
    let guard = match game_storage_locker.read() {
        Err(_) => {
            panic!("Failed to get lock");
        },
        Ok(guard) => {
            guard
        }
    };

    let game_storage = &*guard;

    Some(game_storage.games.get(&id)?.clone())
}

#[get("/new")]
async fn new(data: web::Data<Arc<RwLock<GamesStorage>>>) -> impl Responder {
    let id = {
        let mut guard = match data.get_ref().write() {
            Ok(guard) => guard,
            Err(_) => {
                panic!("Failed to get lock");
            },
        };

        let game_storage = &mut *guard;

        let id = game_storage.sequence;

        game_storage.games.insert(
            id,
            Arc::new(RwLock::new(GoGameEngine::new(BOARD_SIZE))),
        );

        game_storage.sequence += 1;

        id
    };
    HttpResponse::Ok()
        .body(format!("{}", id))
}


#[get("/board/{id}")]
async fn board(info: web::Path<u64>, data: web::Data<Arc<RwLock<GamesStorage>>>) -> impl Responder {
    let go_game = get_game(data.get_ref(), *info);

    if let None = go_game {
        return HttpResponse::NotFound()
            .body(format!("{} is not found", *info));
    }

    let go_game = go_game.unwrap();

    let go_game_guard = match go_game.read() {
        Err(_) => panic!("Failed to get lock!"),
        Ok(guard) => guard,
    };

    let go_game_engine = &*go_game_guard;

    HttpResponse::Ok()
        .body(format!("{}", *go_game_engine))
}

#[get("/make_move/{id}/{x}/{y}")]
//async fn make_move(location: web::Json<Location>) -> impl Responder {
async fn make_move(info: web::Path<(u64, u8, u8)>, data: web::Data<Arc<RwLock<GamesStorage>>>) -> impl Responder {
    let go_game = get_game(data.get_ref(), info.0);

    if let None = go_game {
        return HttpResponse::NotFound()
            .body(format!("{} is not found", info.0));
    }

    let go_game = go_game.unwrap();

    let mut go_game_guard = match go_game.write() {
        Err(_) => panic!("Failed to get lock!"),
        Ok(guard) => guard,
    };

    let go_game_engine = &mut *go_game_guard;

    go_game_engine.make_move(go_board::Location{
        x: info.1,
        y: info.2,
    });

    HttpResponse::Ok()
        .body(format!("{}", info.0))
}

#[get("/close")]
async fn close(info: web::Path<u64>, data: web::Data<Arc<RwLock<GamesStorage>>>) -> impl Responder {
    let mut guard = match data.get_ref().write() {
        Ok(guard) => guard,
        Err(_) => {
            panic!("Failed to get lock");
        },
    };

    let game_storage = &mut *guard;
    let id = *info;

    game_storage.games.remove(&id);
    HttpResponse::Ok()
        .body(format!("{}", id))

}
*/
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

                        self.go_game.make_move(go_board::Location {
                            x: location.x,
                            y: location.y,
                        });

                        for x in 0..BOARD_SIZE {
                            for y in 0..BOARD_SIZE {
                                let chess = match self.go_game.get_chess(go_board::Location {
                                    x,
                                    y,
                                }) {
                                    go_board::ChessType::Black => 'B',
                                    go_board::ChessType::White => 'W',
                                    go_board::ChessType::None => '0',
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
//            .app_data(data.clone())
//            .service(new)
//            .service(board)
//            .service(make_move)
            .route("/ws/", web::get().to(new_go_game))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
