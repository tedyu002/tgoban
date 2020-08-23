#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Serialize)]
pub struct Location {
  pub alphabet: u8,
  pub digit: u8,
}

#[derive(Deserialize, Serialize)]
pub struct GameInfo {
    pub steps: i32,
    pub playing: char,
    pub komi: f64,
    pub capture: [i32; 2],
    pub scores: (f64, f64),
}

#[derive(Deserialize, Serialize)]
pub enum ChessType {
    None,
    BlackLive,
    BlackDead,
    WhiteLive,
    WhiteDead,
}

#[derive(Deserialize, Serialize)]
pub enum Belong {
    None,
    Black,
    White,
}

#[derive(Deserialize, Serialize)]
#[serde(tag="Action", content="content")]
pub enum Action {
    Refresh,
    Play(Location),
    Back,
    GetSGF,
    Pass,
}

#[derive(Deserialize, Serialize)]
#[serde(tag="Command", content="content")]
pub enum Command {
    Set(Vec<ChessType>),
    SetBelong(Vec<Belong>),
    SetGameInfo(GameInfo),
    Sgf(String),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_value() {
        println!("{:#?}", serde_json::to_string_pretty(&Action::Play(Location{alphabet: 5, digit: 5})).unwrap());
        println!("{:#?}", serde_json::to_string_pretty(&Action::Back).unwrap());
        println!("{:#?}", serde_json::to_string_pretty(&Command::Set(vec!['1','2','3','4','5'])).unwrap());
        println!("{:#?}", serde_json::to_string_pretty(&Command::SetGameInfo(GameInfo {playing: 'B', capture: [1, 3]})).unwrap());
    }
}
