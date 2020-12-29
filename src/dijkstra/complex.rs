use std::{
    cmp::Ordering,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
};

#[derive(Debug, Hash, PartialEq, Clone)]
pub struct Node<T>
where
    T: Hash + PartialEq + Clone,
{
    pub identifier: T,
}

#[derive(Debug, Hash, PartialEq, Clone)]
pub struct NodeWithCost<T, C>
where
    T: Hash + PartialEq + Clone,
    C: PartialOrd + Clone,
{
    pub node: Node<T>,
    pub cost: C,
}

impl<T, C> PartialOrd for NodeWithCost<T, C>
where
    T: Hash + PartialEq + Clone,
    C: PartialOrd + Clone,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return if self.cost > other.cost {
            Some(Ordering::Greater)
        } else if self.cost < other.cost {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        };
    }
}

#[derive(Debug)]
pub struct AdjacencyMatrix<T, C>
where
    T: Hash + PartialEq + Clone,
    C: PartialOrd + Clone,
{
    pub matrix: Arc<Mutex<HashMap<Node<T>, Vec<NodeWithCost<T, C>>>>>,
}

impl<T, C> AdjacencyMatrix<T, C>
where
    T: Hash + PartialEq + Clone,
    C: PartialOrd + Clone,
{
    pub fn new() -> Self {
        let matrix = Arc::new(Mutex::new(HashMap::new()));
        return Self { matrix };
    }
}
