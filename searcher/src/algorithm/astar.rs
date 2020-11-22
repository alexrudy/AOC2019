use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;
use std::fmt::Debug;

use super::SearchAlgorithm;
use crate::algorithm::SearchQueue;
use crate::traits::SearchHeuristic;

#[derive(Debug, Eq, PartialEq)]
struct Heuristic<S>
where
    S: Eq,
{
    candidate: S,
}

impl<S> Ord for Heuristic<S>
where
    S: SearchHeuristic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.candidate
            .heuristic()
            .cmp(&other.candidate.heuristic())
            .reverse()
    }
}

impl<S> PartialOrd for Heuristic<S>
where
    S: SearchHeuristic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S> Default for AStarQueue<S>
where
    S: SearchHeuristic,
{
    fn default() -> Self {
        AStarQueue {
            queue: BinaryHeap::new(),
        }
    }
}

#[derive(Debug)]
pub struct AStarQueue<S>
where
    S: SearchHeuristic,
{
    queue: BinaryHeap<Heuristic<S>>,
}

impl<S> SearchQueue for AStarQueue<S>
where
    S: SearchHeuristic,
{
    type Candidate = S;

    fn pop(&mut self) -> Option<Self::Candidate> {
        self.queue.pop().map(|h| h.candidate)
    }

    fn push(&mut self, item: Self::Candidate) {
        self.queue.push(Heuristic { candidate: item });
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

pub type AStarSearcher<S> = SearchAlgorithm<S, AStarQueue<S>>;

pub fn astar<S>(origin: S) -> AStarSearcher<S>
where
    S: SearchHeuristic,
{
    SearchAlgorithm::new(origin)
}
