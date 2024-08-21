use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Peer {
    pub label: u32,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    Ok,
    Ping,
    Pong,
    PeerInfo(Vec<Peer>),

    Data(Vec<u8>),
}
