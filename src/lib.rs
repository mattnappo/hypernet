use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task::spawn;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    io::Read,
    net::SocketAddr,
    process::Command,
    sync::{Arc, Mutex},
};

const NODE_BINARY: &str = "./target/debug/hypernode";

mod util {
    pub fn get_available_ports(n: u16) -> Option<Vec<u16>> {
        let ports = (8000..(12000))
            .filter(|port| port_is_available(*port))
            .take(n as usize)
            .collect::<Vec<u16>>();

        match ports.len() == 4 {
            true => Some(ports),
            _ => None,
        }
    }

    fn port_is_available(port: u16) -> bool {
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// A request to a `Hypernode` on the network
#[derive(Serialize, Deserialize)]
enum Request {
    /// An arbitrary network message
    Message(u32),

    //Broadcast(T),
    //AllGather(T),
    //Scatter(T), // etc...
    /// Request the value of a node
    Value,
    // Request that the node shuts down
    //Shutdown,
}

/// The identity of a `Hypernode`
#[derive(Debug)]
pub struct Identity {
    id: u16,
    address: SocketAddr,
}

impl Identity {
    pub fn new(id: u16, address: SocketAddr) -> Self {
        Self { address, id }
    }
}

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.address == other.address
    }
}

impl Eq for Identity {}

#[derive(Debug)]
/// A `Hypernode` is a standalone process (started with Command::new()) that
/// listens for requests from other `Hypernode`s as well as the client.
pub struct Hypernode {
    /// This node's identity
    identity: Identity,

    /// The data stored locally in this node
    //data: Arc<Mutex<u32>>, // TODO
    data: u32,

    /// Dimension
    d: u16,
}

impl Hash for Identity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for Hypernode {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for Hypernode {}

impl Hypernode {
    pub fn new(identity: Identity, d: u16) -> Self {
        Self {
            identity,
            //data: Arc::new(Mutex::new(0)), // TODO
            data: 0,
            d,
        }
    }

    /// Start listening for requests from other nodes, and requests from
    /// the client (a `Hypercube` struct)
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        let datac = self.data.clone();
        let dc = self.d.clone();
        // Needs to listen for connections and handle them
        TcpListener::bind(self.identity.address)
            .await?
            .incoming()
            .for_each_concurrent(None, |stream| async move {
                Self::handle(stream.unwrap(), datac, dc).await.unwrap(); // TODO: use spawn here
            })
            .await;

        //Ok::<(), std::io::Error>(())
        Ok(())
    }

    /// Handle an incoming request. Determine if its from another node, or the
    /// central client
    async fn handle(mut stream: TcpStream, data: u32, d: u16) -> Result<(), std::io::Error> {
        let mut buffer = [0u8; 1024];
        stream.read(&mut buffer).await.unwrap();
        println!("handling request: {buffer:?}");

        // Decode and handle the request
        let request = bincode::deserialize(&buffer).unwrap();
        match request {
            Request::Message(m) => {
                println!("got message {m:?}");
                0
            }
            Request::Value => stream
                .write(&bincode::serialize(&Request::Message(data)).unwrap())
                .await
                .unwrap(),
        };

        Ok(())
    }
}

/// A d-dimensional cube
/// Really, there is no point in building the `graph` parameter explicitly,
/// since `Hypernode`s will always be able to compute this given `d`. It was fun tho.
#[derive(Debug)]
pub struct Hypercube {
    /// Connections between nodes
    graph: HashMap<u16, HashSet<Identity>>, // TODO Remove eventually

    /// Map from node id to (id, address)
    addrs: HashMap<u16, Identity>,

    /// Dimension of the `Hypercube`
    d: u16,

    /// PIDs of running `Hypernode`s in this `Hypercube`
    pids: Option<Vec<u32>>,
}

impl Hypercube {
    /// Make a `Hypercube` of dimension `d`
    pub fn new(d: u16) -> Self {
        let n = 2u16.pow(d.into());

        // Generate free system addresses, which will then be assigned to nodes
        let addrs = util::get_available_ports(n as u16)
            .expect("unable to find enough ports")
            .into_iter()
            .map(|port| format!("127.0.0.1:{port}").parse().unwrap())
            .collect::<Vec<_>>();

        // Make the Hypercube id mapping
        // Map from id to Set(ident) of adjacent nodes, where ident is (id,addr) pair
        let graph: HashMap<u16, HashSet<Identity>> = (0..n)
            .map(|i| {
                (
                    i as u16,
                    (0..d)
                        .map(|k| {
                            let id = (i as u16) ^ 2u16.pow(k.into());
                            Identity::new(id, addrs[id as usize])
                        })
                        .collect::<HashSet<Identity>>(),
                )
            })
            .collect();

        Self {
            graph,
            addrs: addrs
                .into_iter()
                .enumerate()
                .map(|(i, addr)| (i as u16, Identity::new(i as u16, addr)))
                .collect(),
            d,
            pids: None,
        }
    }

    /// Start the hypercube. Spin up 2**n `Node` processes and make them
    /// start listening for requests. Returns the PIDs of the started node
    /// processes
    pub fn start(&mut self) -> &[u32] {
        let pids = self
            .addrs
            .iter()
            .map(|(_, identity)| {
                let args = [identity.id, self.d, identity.address.port()].map(|n| n.to_string());
                Command::new(NODE_BINARY).args(args).spawn().unwrap().id()
            })
            .collect::<Vec<u32>>();
        self.pids = Some(pids);
        self.pids.as_ref().unwrap()
    }

    /// Query the network to get the current value in every node.
    /// Naive implementation of allgather
    pub fn query(&self) -> Result<HashMap<u16, u32>, std::io::Error> {
        let values = self
            .addrs
            .iter()
            .map(|(id, identity)| {
                // TODO: async this
                let mut stream = std::net::TcpStream::connect(identity.address).unwrap();
                let mut buf = [0u8; 1024];
                stream.read(&mut buf).unwrap();
                let res: Request = bincode::deserialize(&buf).unwrap();
                match res {
                    Request::Message(m) => (id.clone(), m),
                    _ => unreachable!(),
                }
            })
            .collect();
        Ok(values)
    }

    /// A network test function. Each node initially stores 0.
    /// All nodes listen for requests
    /// Upon a node getting a request from another node with message p:
    /// 1. If current (receiving node) value is 0:
    ///    (i ) Send each neighbor the nodes current value plus p + 1
    ///    (ii) Set its current value to current_val + p +1 (which is just p+1)
    /// 2. If current value is not 0:
    ///    (i) Set its current value to current_val + p
    /// This process starts off by the main client sending a request to the 0
    /// node with value 0
    pub fn monotonic(&mut self) {}

    /// Broadcast `value` to all nodes starting from node with id `from`
    pub fn broadcast<T>(&self, from: u32, value: T) {}
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let cube: Hypercube = Hypercube::new(3);

        println!("{cube:#?}");

        let m: HashMap<u16, HashSet<u16>> = cube
            .graph
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|a| a.id).collect::<HashSet<u16>>()))
            .collect();
        assert_eq!(
            m,
            HashMap::from([
                (0b000, HashSet::from([0b001, 0b010, 0b100])),
                (0b001, HashSet::from([0b000, 0b011, 0b101])),
                (0b010, HashSet::from([0b011, 0b000, 0b110])),
                (0b011, HashSet::from([0b010, 0b001, 0b111])),
                (0b100, HashSet::from([0b101, 0b110, 0b000])),
                (0b101, HashSet::from([0b100, 0b111, 0b001])),
                (0b110, HashSet::from([0b111, 0b100, 0b010])),
                (0b111, HashSet::from([0b110, 0b101, 0b011])),
            ])
        );
    }
}
