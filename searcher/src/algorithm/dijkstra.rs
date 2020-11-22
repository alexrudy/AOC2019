use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;
use std::fmt::Debug;

use super::SearchAlgorithm;
use crate::algorithm::SearchQueue;
use crate::traits::SearchCandidate;

#[derive(Debug, Eq, PartialEq)]
pub struct DjirkstraElement<S> {
    element: S,
}

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

pub type DijkstraSearch<S> = SearchAlgorithm<S, DijkstraQueue<S>>;

pub fn djirkstra<S>(origin: S) -> DijkstraSearch<S>
where
    S: SearchCandidate,
{
    SearchAlgorithm::new(origin)
}
