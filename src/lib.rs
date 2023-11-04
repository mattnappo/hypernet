use std::collections::{HashMap, HashSet};

/// A d-dimensional cube
#[derive(Debug)]
struct Hypercube {
    /// Connections between nodes
    graph: HashMap<u32, HashSet<u32>>,

    /// Dimension of the hypercube
    d: u32,

    /// Number of nodes: 2**d
    n: u32,
}

impl Hypercube {
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
                    .insert(i ^ 2u32.pow(k));
            })
        });

        Self {
            graph,
            d,
            n: 2u32.pow(d),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let cube = Hypercube::new(3);
        assert_eq!(
            cube.graph,
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
