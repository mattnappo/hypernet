#![allow(dead_code)]

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use anyhow::{Context, Result};
use local_ip_address::local_ip;
use log::{debug, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::protocol::{Message, Peer};

pub struct Identity {
    ip: IpAddr,
    port: u16,
}

impl Identity {
    pub fn new(ip: IpAddr, port: u16) -> Identity {
        Identity { ip, port }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

// type Label = u32;

// TODO: Add a PhantomData here so that we can mark a node as ready for use or not.
// I.e. can't start a node when label is None or neighbors is empty.
// TODO: Make this a lifetime so that identity is borrowed
pub struct Node {
    label: Option<u32>,
    /// Peer label to Identity map
    peers: HashMap<u32, Identity>,
    identity: Identity,
}

impl Node {
    pub fn new(port: u16) -> Result<Self> {
        let ip = local_ip().context("could not resolve local ip")?;
        Ok(Node {
            label: None,
            peers: HashMap::new(),
            identity: Identity::new(ip, port),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let ip = format!("0.0.0.0:{}", self.identity.port);
        let server = TcpListener::bind(&ip).await?;
        info!("node started listening on {ip}");

        loop {
            let (mut socket, addr) = server.accept().await?;
            let t0 = std::time::Instant::now();
            info!("accepted new connection from {addr:?}");
            // Read from client
            let mut buf = [0; 1024];
            let len = socket.read(&mut buf).await?;
            debug!("read {} bytes: {:?}", len, &buf[..len]);
            let req: Message = rmp_serde::from_slice(&buf[0..len])?;
            info!("handling request {:?} from {addr:?}", req);

            let res = match req {
                Message::Ping => Message::Pong,
                Message::PeerInfo(peers) => {
                    for Peer { label, ip, port } in peers {
                        self.peers.insert(label, Identity { ip, port });
                    }
                    Message::Ok
                }
                _ => unimplemented!(),
            };
            socket.write_all(&rmp_serde::to_vec(&res)?).await?;
            info!(
                "wrote {res:?} to {addr:?} in {:?}us",
                std::time::Instant::elapsed(&t0).as_micros()
            );
        }

        Ok(())
    }
}

/// Request handlers.
trait Server {
    //async fn handle_ping(&self);
    //async fn set_peers(&mut self);
}

/// Request senders for point-to-point communications.
trait Client {
    async fn send_ping(&self, peer: &Identity) -> Result<Message>;
    async fn set_peers(&mut self, peer: &Identity, peers: Vec<Peer>) -> Result<Message>;
}

impl Server for Node {
    // async fn ping(&self) {}
    //async fn set_peers(&mut self) {}
}

// TODO: make a helper for these like TaskRpc
impl Client for Node {
    async fn send_ping(&self, peer: &Identity) -> Result<Message> {
        send_helper(peer, &Message::Ping).await
    }

    async fn set_peers(&mut self, peer: &Identity, peers: Vec<Peer>) -> Result<Message> {
        send_helper(peer, &Message::PeerInfo(peers)).await
    }
}

async fn send_helper(peer: &Identity, req: &Message) -> Result<Message> {
    // Connect, write req, and read res
    let mut conn = TcpStream::connect(peer.address()).await?;
    conn.write_all(&rmp_serde::to_vec(req)?).await?;
    let mut buf = [0; 1024];
    let len = conn.read(&mut buf).await?;
    let res: Message = rmp_serde::from_slice(&buf[0..len])?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests assume that there is a node already listening on 8080.
    const TEST_IP: &str = "0.0.0.0";
    const TEST_PORT: u16 = 8080;

    fn identity() -> Identity {
        Identity {
            ip: TEST_IP.parse().unwrap(),
            port: TEST_PORT,
        }
    }

    #[tokio::test]
    async fn test_send_ping() -> Result<()> {
        let node_client = Node::new(9090)?;
        let res = node_client.send_ping(&identity()).await?;
        assert_eq!(res, Message::Pong);
        Ok(())
    }

    #[tokio::test]
    async fn test_send_peer() -> Result<()> {
        let mut node_client = Node::new(9090)?;
        let peers = vec![Peer {
            label: 100,
            ip: "127.0.0.1".parse()?,
            port: 9000,
        }];
        let res = node_client.set_peers(&identity(), peers).await?;
        assert_eq!(res, Message::Ok);
        Ok(())
    }
}
