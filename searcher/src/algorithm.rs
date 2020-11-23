use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;

use self::cache::Cache;
use crate::errors::{Result, SearchError};
use crate::traits::SearchCandidate;

pub(crate) mod astar;
pub(crate) mod basic;
pub(crate) mod cache;
pub(crate) mod dijkstra;

/// Trait used to implement queues of search candidates
/// which should be checked for completion.
pub trait SearchQueue {
    type Candidate;

    fn pop(&mut self) -> Option<Self::Candidate>;

    fn push(&mut self, item: Self::Candidate);

    fn len(&self) -> usize;
}

#[derive(Debug, Eq, PartialEq)]
struct Score<S>
where
    S: SearchCandidate,
{
    candidate: S,
}

impl<S> Score<S>
where
    S: SearchCandidate,
{
    fn new(candidate: S) -> Self {
        Self { candidate }
    }
}

impl<S> Ord for Score<S>
where
    S: SearchCandidate,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.candidate
            .score()
            .cmp(&other.candidate.score())
            .reverse()
    }
}

impl<S> PartialOrd for Score<S>
where
    S: SearchCandidate,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct StepLimit {
    current: usize,
    maximum: usize,
}

impl StepLimit {
    fn new(limit: usize) -> Self {
        Self {
            current: 0,
            maximum: limit,
        }
    }

    fn increment(&mut self) -> Result<()> {
        self.current += 1;

        if self.current >= self.maximum {
            Err(SearchError::StepLimitExhausted(self.current))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default)]
pub struct SearchAlgorithm<S, Q, C>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
    C: Cache<Candidate = S>,
{
    cache: C,
    queue: Q,
    results: BinaryHeap<Score<S>>,
    counter: Option<StepLimit>,
}

impl<S, Q, C> SearchAlgorithm<S, Q, C>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
    C: Cache<Candidate = S>,
{
    fn new(origin: S) -> Self {
        let mut sr = SearchAlgorithm {
            cache: C::default(),
            queue: Q::default(),
            results: BinaryHeap::default(),
            counter: None,
        };
        sr.queue.push(origin);
        sr
    }
    pub fn set_limit(&mut self, limit: usize) {
        self.counter = Some(StepLimit::new(limit))
    }

    fn best(&self) -> Option<&S> {
        self.results.peek().map(|s| &s.candidate)
    }

    // Should we continue searching from this candidate?
    fn process_candidate(&mut self, candidate: &S) -> Result<bool> {
        // Increment the step counter
        self.counter
            .as_mut()
            .map(|c| c.increment())
            .unwrap_or(Ok(()))?;

        // If we found an answer, we can stop hunting now
        // and add the answer to our search results.
        if candidate.is_complete() {
            self.results.push(Score::new(candidate.clone()));
            return Ok(false);
        }

        // Scores can only increase in searches, if the best candidate
        // is better than our current guess, give up now.
        let score = candidate.score();
        if score > self.best().map(|s| s.score()).unwrap_or(usize::MAX) {
            return Ok(false);
        }

        self.cache.check(candidate)
    }

    pub fn run(mut self) -> Result<S> {
        let mut n = 0;
        while let Some(candidate) = self.queue.pop() {
            n += 1;

            let will_process = self.process_candidate(&candidate)?;
            if n % 10_000 == 0 {
                eprintln!(
                    "Q{} R{} S{:?} ({} {}) {}",
                    self.queue.len(),
                    self.results.len(),
                    self.best().map(|p| p.score()),
                    candidate.score(),
                    if will_process { "y" } else { "n" },
                    n
                );
            }

            if will_process {
                for child in candidate.children() {
                    self.queue.push(child);
                }
            }
        }
        self.best().cloned().ok_or(SearchError::NoResultFound)
    }
}
