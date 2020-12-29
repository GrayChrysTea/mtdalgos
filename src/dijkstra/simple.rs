//! Simple version of the Dijkstra Algorithm, which only allows you to use
//! [`usize`] to idenfity nodes and [`u128`] to represent the costs to go
//! between 2 nodes.

//#![allow(unused_imports)]
use std::{
    collections::{BinaryHeap, HashMap},
    cmp::{Ordering, max},
    io::{Error, ErrorKind},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use crate::{
    pool::ThreadPool,
    unwrapoption,
    unwrapmutex,
};

/// Custom wrapper type around [`u128`]. This type is used to represent the
/// cost to get from one node to another node. Due to the nature of Dijkstra
/// and it's inefficiency with negative integers, an unsigned integer is used
/// instead.
pub type Cost = u128;

/// Identifier for a node in the graph. [`usize`] is used to identify it.
pub type Node = usize;

/// A custom struct to represent a destination [`Node`] and the [`Cost`] to
/// reach it from an arbitrary starting point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeWithCost {
    pub node: Node,
    /// u128 since dijkstra doesn't handle negatives too well
    pub cost: Cost,
}

impl NodeWithCost {
    /// Creates a new [`NodeWithCost`].
    pub fn new(node: Node, cost: Cost) -> Self {
        return Self {node, cost};
    }
}

impl PartialOrd for NodeWithCost {
    /// This function marks a greater cost as [`Ordering::Less`] and vice versa
    /// for [`Ordering::Greater`] to trick the BinaryHeap into floating the
    /// cheaper nodes to the top.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;
        return if self.cost > other.cost {
            Some(Less)
        } else if self.cost < other.cost {
            Some(Greater)
        } else {
            Some(Equal)
        };
    }
}

impl Ord for NodeWithCost where Self: PartialOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.partial_cmp(other).unwrap();
    }
}

/// An adjacency matrix which represents the graph. The first [`Vec`]tor
/// represents each [`Node`] present as a starting point, with all neighbouring
/// [`Node`]s in the [`Vec`] inside it.
/// 
/// An array is not used because Rust requires the size of the array to be
/// known at compile time. This is not necessary with [`Vec`]tors.
#[derive(Debug)]
pub struct AdjacencyMatrix {
    matrix: Vec<Vec<NodeWithCost>>,
}

impl AdjacencyMatrix {
    /// Creates a new [`AdjacencyMatrix`] with a fixed amount of [`Node`]s.
    pub fn new(total: Node) -> Self {
        let mut matrix: Vec<Vec<NodeWithCost>> = Vec::with_capacity(total);
        matrix.resize(total, Vec::new());
        return Self {matrix};
    }

    /// Pushes an adjacent [`Node`] and the [`Cost`] to reach it (as a
    /// [`NodeWithCost`]) to an origin [`Node`].
    /// 
    /// If the destination [`Node`] is already added to the origin [`Node`],
    /// the cheaper route (i.e. the `to` with the lower [`Cost`]) is used as
    /// the route used for calculations.
    /// 
    /// If `from` or `to.node` exceeds the length of the matrix, an error is
    /// returned.
    pub fn push(&mut self, from: Node, to: NodeWithCost) -> Result<(), Error> {
        if from >= self.matrix.len() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{} exceeds matrix size.", from)
            ));
        } else if to.node >= self.matrix.len() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{} exceeds matrix size.", to.node)
            ));
        }
        if from == to.node {
            return Ok(());
        }
        let target = match self.matrix.get_mut(from) {
            Some(adjacents) => adjacents,
            None => return Err(Error::new(
                ErrorKind::AddrNotAvailable,
                "Could not access matrix."
            ))
        };
        let mut found = false;
        for existing in target.iter_mut() {
            if existing.node == to.node {
                existing.cost = max(existing.cost, to.cost);
                found = true;
                break;
            }
        }
        if !found {
            target.push(to);
        }
        return Ok(());
    }

    /// Get the number of [`Node`]s in the graph.
    pub fn total(&self) -> Node {
        return self.matrix.len();
    }

    /// Get the adjacent [`Node`]s from a starting node.
    pub fn get_node(&self, node: Node) -> Option<&Vec<NodeWithCost>> {
        return self.matrix.get(node);
    }
}

/// This `struct` contains the implementations to calculate the shortest route
/// from [`Node`] in the graph using multiple threads.
pub struct MtdDijkstra {
    pool: ThreadPool,
    costs: Arc<Mutex<HashMap<Node, Vec<Option<Cost>>>>>,
    nodes: Node,
    matrix: Arc<Mutex<AdjacencyMatrix>>,
}

impl MtdDijkstra {
    /// Creates a new [`MtdDijkstra`] instance.
    /// 
    /// # Parameters
    /// 1. ```threads: usize``` => Number of threads to use. At least one
    /// thread is needed to run the algorithm.
    /// 2. ```nodes: Node``` => Number of nodes in the graph.
    /// 3. ```matrix: AdjacencyMatrix``` => The adjacency matrix which
    /// describes the graph.
    /// 
    /// # Error
    /// 
    /// This function will return a [`std::io::Error`] if `threads` is less
    /// than `1`.
    pub fn new(
        threads: usize,
        nodes: Node,
        matrix: AdjacencyMatrix
    ) -> Result<Self, Error> {
        let pool = ThreadPool::new(threads)?;
        let costs: Arc<Mutex<HashMap<Node, Vec<Option<Cost>>>>> = Arc::new(
            Mutex::new(HashMap::new())
        );
        let matrix = Arc::new(Mutex::new(matrix));
        return Ok(Self {pool, costs, nodes, matrix});
    }

    /// Calculates the shortest distance to all (if possible) nodes in the
    /// graph from each node. This method uses a [`ThreadPool`] to run the
    /// algorithm. If something wrong happens, a [`std::io::Error`] is
    /// returned.
    pub fn calculate(&mut self) -> Result<(), Error> {
        for node in 0..self.nodes {
            let nodes = self.nodes;
            let node = node;
            let matrix = self.matrix.clone();
            let costs = self.costs.clone();
            self.pool.execute(move || {

                let mut distances: Vec<Option<Cost>> = Vec::with_capacity(
                    nodes
                );
                // Set everything to unvisited
                distances.resize(nodes, None);
                // Set starting node to 0
                *unwrapoption!(distances.get_mut(node)) = Some(0);
                
                let mut unvisited: BinaryHeap<NodeWithCost> = BinaryHeap::new();
                unvisited.push(NodeWithCost::new(node, 0));

                while !unvisited.is_empty() {
                    if let Some(current) = unvisited.pop() {
                        if Some(current.cost) != *unwrapoption!(
                            distances.get(current.node)
                        ) {
                            continue;
                        }
                        for adjacent in unwrapoption!(
                            unwrapmutex!(matrix.lock())
                                .matrix
                                .get(current.node)
                        ) {
                            let adjacent_distance = unwrapoption!(
                                distances.get_mut(adjacent.node)
                            );
                            if let Some(distance) = adjacent_distance {
                                let mut new_distance = *distance;
                                if *distance > current.cost + adjacent.cost {
                                    *distance = current.cost + adjacent.cost;
                                    new_distance = *distance;
                                }
                                unvisited.push(NodeWithCost::new(
                                    adjacent.node,
                                    new_distance,
                                ));
                            } else if let None = adjacent_distance {
                                let new_distance = current.cost+adjacent.cost;
                                *adjacent_distance = Some(new_distance);
                                unvisited.push(NodeWithCost::new(
                                    adjacent.node,
                                    new_distance,
                                ));
                            }
                        }
                    } else {
                        break;
                    }
                }

                let mut inner_cost = unwrapmutex!(costs.lock());
                inner_cost.insert(node, distances);
                
                return Ok(());
            })?;
        }
        return Ok(());
    }

    /// Get the inner cost [`std::collections::HashMap`].
    pub fn get_result(self) -> Arc<Mutex<HashMap<Node, Vec<Option<Cost>>>>> {
        return self.costs;
    }

    /// Get a copy of the cost to get to all destination [`Node`]s from one
    /// starting [`Node`].
    /// 
    /// Since 0.2: Blocks until all [`Node`]s have been calculated.
    pub fn get(&mut self, node: Node) -> Option<Vec<Option<Cost>>> {
        let mut jobs_ok: usize = 0;
        let mut jobs_err: usize = 0;
        while jobs_ok < self.nodes && jobs_err == 0 {
            sleep(Duration::from_millis(50));
            jobs_ok = self.pool.jobs_ok().ok()?;
            jobs_err = self.pool.jobs_err().ok()?;
        }
        if jobs_err > 0 {
            return None;
        }
        let costs = match self.costs.lock() {
            Ok(costs) => costs,
            Err(_error) => return None,
        }.get(&node)?.clone();
        return Some(costs);
    }
}