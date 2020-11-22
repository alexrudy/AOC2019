use std::collections::VecDeque;
use std::default::Default;
use std::fmt::Debug;

use crate::algorithm::SearchQueue;
use crate::traits::SearchCandidate;

use super::SearchAlgorithm;

#[derive(Debug)]
pub struct BreadthQueue<S> {
    queue: VecDeque<S>,
}

impl<S> SearchQueue for BreadthQueue<S> {
    type Candidate = S;

    fn pop(&mut self) -> Option<Self::Candidate> {
        self.queue.pop_front()
    }

    fn push(&mut self, item: Self::Candidate) {
        self.queue.push_back(item);
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

impl<S> Default for BreadthQueue<S> {
    fn default() -> Self {
        BreadthQueue {
            queue: VecDeque::new(),
        }
    }
}

#[derive(Debug)]
pub struct DepthQueue<S> {
    queue: VecDeque<S>,
}

impl<S> Default for DepthQueue<S> {
    fn default() -> Self {
        DepthQueue {
            queue: VecDeque::new(),
        }
    }
}

impl<S> SearchQueue for DepthQueue<S> {
    type Candidate = S;

    fn pop(&mut self) -> Option<Self::Candidate> {
        self.queue.pop_front()
    }

    fn push(&mut self, item: Self::Candidate) {
        self.queue.push_front(item);
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

pub type BreadthFirstSearcher<S> = SearchAlgorithm<S, BreadthQueue<S>>;

pub fn bfs<S>(origin: S) -> BreadthFirstSearcher<S>
where
    S: SearchCandidate,
{
    SearchAlgorithm::new(origin)
}

pub type DepthFirstSearcher<S> = SearchAlgorithm<S, DepthQueue<S>>;

pub fn dfs<S>(origin: S) -> DepthFirstSearcher<S>
where
    S: SearchCandidate,
{
    SearchAlgorithm::new(origin)
}
