use serde::Deserialize;

#[derive(Deserialize, Serialize)]
pub struct Location {
  pub alphabet: u8,
  pub digit: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(tag="Action", content="content")]
pub enum Action {
    Play(Location),
    Back,
}

#[derive(Serialize)]
#[serde(tag="Command", content="content")]
pub enum Command {
    Set(Vec<char>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_value() {
        println!("{:#?}", serde_json::to_string_pretty(&Action::Play(Location{alphabet: 5, digit: 5})).unwrap());
        println!("{:#?}", serde_json::to_string_pretty(&Action::Back).unwrap());
        println!("{:#?}", serde_json::to_string_pretty(&Command::Set(vec!['1','2','3','4','5'])).unwrap());
    }
}
