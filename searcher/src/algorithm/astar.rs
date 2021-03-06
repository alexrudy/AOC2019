//! A* Search Algorithm

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;
use std::fmt::Debug;

use super::cache::BasicCache;
use super::SearchAlgorithm;
use crate::errors::Result;
use crate::traits::{SearchCacher, SearchHeuristic};
use crate::{algorithm::SearchQueue, SearchCandidate};

#[derive(Debug)]
struct Heuristic<S>
where
    S: SearchHeuristic,
{
    heuristic: S::Hueristic,
    candidate: S,
}

impl<S> PartialEq for Heuristic<S>
where
    S: SearchHeuristic,
{
    fn eq(&self, other: &Self) -> bool {
        self.heuristic.eq(&other.heuristic)
    }
}

impl<S> Eq for Heuristic<S> where S: SearchHeuristic {}

impl<S> Ord for Heuristic<S>
where
    S: SearchHeuristic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.heuristic.cmp(&other.heuristic).reverse()
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

/// A priority queue which always returns the best-guess
/// search candidate based on a heuristic from [SearchHeuristic]
/// and so implements A* search.
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
        self.queue.push(Heuristic {
            heuristic: item.heuristic(),
            candidate: item,
        });
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

type AStarSearcher<S> = SearchAlgorithm<S, AStarQueue<S>, BasicCache<S>>;

pub fn build<S>(origin: S) -> AStarSearcher<S>
where
    S: SearchHeuristic + SearchCandidate + SearchCacher + Ord,
{
    SearchAlgorithm::new(origin)
}

/// Perform a search using the A* algorithm, which leverages a heuristic provided by [SearchHeuristic].
///
/// A* always considers the next candidate to be the one with the lowest
/// estimated score, as provided by the heuristic.
pub fn run<S>(origin: S) -> Result<S>
where
    S: SearchHeuristic + SearchCandidate + SearchCacher + Ord,
{
    build(origin).run()
}
