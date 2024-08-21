use std::collections::HashMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// Size of the largest message that can be sent over the network.
pub const MAX_MESSAGE_SIZE: usize = 1024;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Peer {
    pub label: u32,
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    Ok,
    Err(String),
    Ping,
    Pong,

    SetPeerInfo(Vec<Peer>),
    GetPeerInfo,
    PeerInfo(HashMap<u32, Identity>),

    Data(Vec<u8>),
}

impl Identity {
    pub fn new(ip: IpAddr, port: u16) -> Identity {
        Identity { ip, port }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}
