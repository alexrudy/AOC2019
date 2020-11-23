use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;
use std::fmt::Debug;

use super::{cache::BasicCache, SearchAlgorithm};
use crate::algorithm::SearchQueue;
use crate::errors::Result;
use crate::traits::{SearchCacher, SearchCandidate};

#[derive(Debug)]
pub struct DjirkstraElement<S>
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
}

pub type DijkstraSearch<S> = SearchAlgorithm<S, DijkstraQueue<S>, BasicCache<S>>;

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
pub fn dijkstra<S>(origin: S) -> Result<S>
where
    S: SearchCandidate + SearchCacher,
{
    build(origin).run()
}
