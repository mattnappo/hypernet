use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
struct Hypernode<T: Default> {
    address: Option<Ipv4Addr>,
    id: u32,
    data: Arc<Mutex<T>>,
}

impl<T: Default> Hash for Hypernode<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl<T: Default> PartialEq for Hypernode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Default> Eq for Hypernode<T> {}

impl<T: Default> Hypernode<T> {
    pub fn new(id: u32) -> Self {
        Self {
            address: None,
            id,
            data: Arc::new(Mutex::new(T::default())),
        }
    }

    pub fn start() {
        // Needs to listen for connections and handle them
    }
}

/// A d-dimensional cube
#[derive(Debug)]
struct Hypercube<T: Default> {
    /// Connections between nodes
    graph: HashMap<u32, HashSet<Hypernode<T>>>,

    /// Dimension of the hypercube
    d: u32,

    /// Number of nodes: 2**d
    n: u32,
}

impl<T: Default> Hypercube<T> {
    /// Make a `Hypercube` of dimension `d`
    pub fn new(d: u32) -> Self {
        // Make the Hypercube id mapping
        let mut graph = HashMap::new();
        //<u32, Vec<u32>>
        (0..(2u32.pow(d))).for_each(|i| {
            (0..d).for_each(|k| {
                graph
                    .entry(i)
                    .or_insert(HashSet::new())
                    .insert(Hypernode::new(i ^ 2u32.pow(k)));
            })
        });

        Self {
            graph,
            d,
            n: 2u32.pow(d),
        }
    }

    /// Send data to all nodes, from an id
    pub fn broadcast(&self, from: u32, data: T) {}
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let cube: Hypercube<()> = Hypercube::new(3);

        let m: HashMap<u32, HashSet<u32>> = cube
            .graph
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|a| a.id).collect::<HashSet<u32>>()))
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
