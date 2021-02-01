pub use bincode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    GetGlobalHangulState,
    UpdateHangulState(bool),
}

#[derive(Serialize, Deserialize)]
pub struct GetGlobalHangulStateReply {
    pub state: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
