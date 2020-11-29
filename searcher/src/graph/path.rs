use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Sum;

use super::Edge;

#[derive(Debug, Clone)]
pub struct GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Node = N>,
{
    pub(crate) nodes: Vec<N>,
    pub(crate) edges: Vec<E>,
}

impl<N, E> GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Node = N>,
{
    pub fn new(origin: N) -> Self {
        Self {
            nodes: vec![origin],
            edges: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn step_one(&self, node: N, edge: E) -> Self {
        let mut nextpath = self.clone();
        nextpath.nodes.push(node);
        nextpath.edges.push(edge);
        nextpath
    }

    pub fn origin(&self) -> &N {
        self.nodes.first().unwrap()
    }

    pub fn destination(&self) -> &N {
        self.nodes.last().unwrap()
    }

    pub(crate) fn penultimate(&self) -> Option<&N> {
        let n = self.nodes.len();
        if n > 1 {
            Some(&self.nodes[n - 2])
        } else {
            None
        }
    }
}

impl<N, E, W> Ord for GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Weight = W, Node = N>,
    W: Sum + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight().cmp(&other.weight())
    }
}

impl<N, E, W> PartialOrd for GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Weight = W, Node = N>,
    W: Sum + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N, E, W> PartialEq for GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Weight = W, Node = N>,
    W: Sum + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.weight().eq(&other.weight())
    }
}

impl<N, E, W> Eq for GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Weight = W, Node = N>,
    W: Sum + Eq,
{
}

impl<N, E, W> GraphPath<N, E>
where
    N: Debug + Clone,
    E: Edge<Weight = W, Node = N>,
    W: Sum,
{
    pub fn weight(&self) -> W {
        self.edges.iter().map(|e| e.weight()).sum()
    }
}

impl<N, E, W> Edge for GraphPath<N, E>
where
    N: Debug + Hash + Clone + Eq,
    E: Edge<Weight = W, Node = N>,
    W: Sum + Ord + Eq + Debug + Copy,
{
    type Weight = W;
    type Node = N;

    fn weight(&self) -> Self::Weight {
        self.edges.iter().map(|e| e.weight()).sum()
    }

    fn new(node: Self::Node) -> Self {
        Self {
            nodes: vec![node],
            edges: Vec::new(),
        }
    }

    fn origin(&self) -> &Self::Node {
        self.nodes.first().unwrap()
    }

    fn destination(&self) -> &Self::Node {
        self.nodes.last().unwrap()
    }

    fn step(&self, node: Self::Node, edge: Self) -> Self {
        let mut nextpath = self.clone();
        nextpath.nodes.push(node);
        nextpath.edges.extend(edge.edges.into_iter());
        nextpath
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
