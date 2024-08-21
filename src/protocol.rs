use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Ping,
    Pong,

    Data(Vec<u8>),
}
