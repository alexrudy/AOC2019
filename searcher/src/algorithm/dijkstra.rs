//! Dijrkstra's Algorithm

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;
use std::fmt::Debug;

use super::{cache::BasicCache, SearchAlgorithm};
use crate::algorithm::SearchQueue;
use crate::errors::Result;
use crate::traits::{SearchCacher, SearchCandidate};

/// Wrapper for search candidates which sorts appropriately
/// for Dijrkstra's Algorithm.
#[derive(Debug)]
struct DjirkstraElement<S>
where
    S: SearchCandidate,
{
    element: S,
}

impl<S> PartialEq for DjirkstraElement<S>
where
    S: SearchCandidate,
{
    fn eq(&self, other: &Self) -> bool {
        self.element.score().eq(&other.element.score())
    }
}

impl<S> Eq for DjirkstraElement<S> where S: SearchCandidate {}

impl<S> Ord for DjirkstraElement<S>
where
    S: SearchCandidate,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.element.score().cmp(&other.element.score()).reverse()
    }
}

impl<S> PartialOrd for DjirkstraElement<S>
where
    S: SearchCandidate,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A priority queue to always search the next shortest path
/// by measured distance.
#[derive(Debug)]
pub struct DijkstraQueue<S>
where
    S: SearchCandidate,
{
    queue: BinaryHeap<DjirkstraElement<S>>,
}

impl<S> Default for DijkstraQueue<S>
where
    S: SearchCandidate,
{
    fn default() -> Self {
        DijkstraQueue {
            queue: BinaryHeap::new(),
        }
    }
}

impl<S> SearchQueue for DijkstraQueue<S>
where
    S: SearchCandidate,
{
    type Candidate = S;

    fn pop(&mut self) -> Option<Self::Candidate> {
        self.queue.pop().map(|h| h.element)
    }

    fn push(&mut self, item: Self::Candidate) {
        self.queue.push(DjirkstraElement { element: item });
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    #[allow(unused_variables)]
    fn can_terminate(&self, candidate: &Self::Candidate) -> bool {
        true
    }
}

/// Search algorithm which implements Dijkstra's Algorithm for
/// graph searches.
pub type DijkstraSearch<S> = SearchAlgorithm<S, DijkstraQueue<S>, BasicCache<S>>;

/// Build a Dijkstra's Alogrithm Searcher
pub fn build<S>(origin: S) -> DijkstraSearch<S>
where
    S: SearchCandidate + SearchCacher,
{
    SearchAlgorithm::new(origin)
}

/// Perform a search using Dijkstra's algorithm.
///
/// Dijkstra's algorithm behaves like a breadth first search, but always
/// searches the next shortest path even when paths end up with varying
/// lenghts. To be optimal, Dijkstra's algorithm requires that it remember
/// the states observed, hence the SearchCacher constraint.
pub fn run<S>(origin: S) -> Result<S>
where
    S: SearchCandidate + SearchCacher,
{
    build(origin).run()
}
