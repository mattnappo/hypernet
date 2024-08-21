#![allow(dead_code)]

use std::net::IpAddr;

use anyhow::{Context, Result};
use local_ip_address::local_ip;
use log::{debug, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::protocol::Message;

pub struct Identity {
    ip: IpAddr,
    port: u16,
}

impl Identity {
    pub fn new(ip: IpAddr, port: u16) -> Identity {
        Identity { ip, port }
    }
}

// TODO: Add a PhantomData here so that we can mark a node as ready for use or not.
// I.e. can't start a node when label is None or neighbors is empty.
// TODO: Make this a lifetime so that identity is borrowed
pub struct Node {
    label: Option<u32>,
    neighbors: Vec<Identity>,
    identity: Identity,
}

impl Node {
    pub fn new(port: u16) -> Result<Self> {
        let ip = local_ip().context("could not resolve local ip")?;
        Ok(Node {
            label: None,
            neighbors: vec![],
            identity: Identity::new(ip, port),
        })
    }

    pub async fn start(&self) -> Result<()> {
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

/// Request senders
trait Client {
    async fn send_ping(&self, ip: &str, port: u16) -> Result<()>;

    //async fn set_peers(&mut self);
}

impl Server for Node {
    // async fn ping(&self) {}
    //async fn set_peers(&mut self) {}
}

impl Client for Node {
    async fn send_ping(&self, ip: &str, port: u16) -> Result<()> {
        let mut conn = TcpStream::connect(format!("{ip}:{port}")).await?;
        conn.write_all(&rmp_serde::to_vec(&Message::Ping)?).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_ping() -> Result<()> {
        // Assumes 8080 is already listening
        let node_client = Node::new(9090)?;
        node_client.send_ping("0.0.0.0", 8080).await?;
        Ok(())
    }
}
