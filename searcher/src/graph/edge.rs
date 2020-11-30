use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::convert::From;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Edge: Debug + PartialOrd + Ord + Clone {
    type Weight: Debug + PartialOrd + Ord + Copy + Clone;
    type Node: Debug + PartialEq + Eq + Hash + Clone;

    fn weight(&self) -> Self::Weight;

    fn new(node: Self::Node) -> Self;

    fn origin(&self) -> &Self::Node;

    fn destination(&self) -> &Self::Node;

    fn reverse(&self) -> Self;

    fn step(&self, node: Self::Node, edge: Self) -> Self;

    fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct WeightedEdge<E>
where
    E: Edge,
{
    pub(crate) edge: E,
    weight: E::Weight,
}

impl<E> WeightedEdge<E>
where
    E: Edge,
{
    pub fn reverse(&self) -> Self {
        let edge = self.edge.reverse();
        edge.into()
    }
}

impl<E> From<E> for WeightedEdge<E>
where
    E: Edge,
{
    fn from(edge: E) -> Self {
        WeightedEdge {
            weight: edge.weight(),
            edge: edge,
        }
    }
}

impl<E> Ord for WeightedEdge<E>
where
    E: Edge,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl<E> PartialOrd for WeightedEdge<E>
where
    E: Edge,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> PartialEq for WeightedEdge<E>
where
    E: Edge,
{
    fn eq(&self, other: &Self) -> bool {
        self.weight.eq(&other.weight)
    }
}

impl<E> Eq for WeightedEdge<E> where E: Edge {}
