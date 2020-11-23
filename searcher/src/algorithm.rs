//! Provides the building blocks for search algorithms

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;

use self::cache::Cache;
use crate::errors::{Result, SearchError};
use crate::traits::SearchCandidate;

pub mod astar;
pub mod basic;
pub mod cache;
pub mod dijkstra;

/// Trait used to implement queues of search candidates
/// which should be checked for completion.
pub trait SearchQueue {
    type Candidate;

    fn pop(&mut self) -> Option<Self::Candidate>;

    fn push(&mut self, item: Self::Candidate);

    fn len(&self) -> usize;

    #[allow(unused_variables)]
    fn can_terminate(&self, candidate: &Self::Candidate) -> bool {
        false
    }
}

#[derive(Debug)]
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

impl<S> PartialEq for Score<S>
where
    S: SearchCandidate,
{
    fn eq(&self, other: &Self) -> bool {
        self.candidate.score().eq(&other.candidate.score())
    }
}

impl<S> Eq for Score<S> where S: SearchCandidate {}

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
pub struct SearchOptions {
    pub limit: Option<usize>,
    pub verbose: Option<usize>,
}

/// Implementation of search, using generic components.
///
/// Uses a generic queue (Q) and a generic cache (C) to provide
/// a single foundation for multiple search algorithms.
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
    options: SearchOptions,
}

impl<S, Q, C> SearchAlgorithm<S, Q, C>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
    C: Cache<Candidate = S>,
{
    fn new_with_options(origin: S, options: SearchOptions) -> Self {
        let counter = options.limit.map(|l| StepLimit::new(l));

        let mut sr = SearchAlgorithm {
            cache: C::default(),
            queue: Q::default(),
            results: BinaryHeap::default(),
            counter: counter,
            options: options,
        };
        sr.queue.push(origin);
        sr
    }

    fn new(origin: S) -> Self {
        Self::new_with_options(origin, SearchOptions::default())
    }

    pub fn with_options(mut self, options: SearchOptions) -> Self {
        let origin = self.queue.pop().unwrap();

        Self::new_with_options(origin, options)
    }

    fn best(&self) -> Option<&S> {
        self.results.peek().map(|s| &s.candidate)
    }

    // Should we continue searching from this candidate?
    fn process_candidate(&mut self, candidate: S) -> Result<Option<S>> {
        // Increment the step counter
        self.counter
            .as_mut()
            .map(|c| c.increment())
            .unwrap_or(Ok(()))?;

        // If we found an answer, we can stop hunting now
        // and add the answer to our search results.
        if candidate.is_complete() {
            self.results.push(Score::new(candidate));
            return Ok(None);
        }

        // Scores can only increase in searches, if the best candidate
        // is better than our current guess, give up now.
        let score = candidate.score();
        if score > self.best().map(|s| s.score()).unwrap_or(usize::MAX) {
            return Ok(None);
        }

        if self.cache.check(&candidate)? {
            return Ok(Some(candidate));
        }
        Ok(None)
    }

    pub fn show_debug_msg(&self, n: usize) -> bool {
        self.options.verbose.map(|v| n % v == 0).unwrap_or(false)
    }

    /// Run the search to completion.
    pub fn run(mut self) -> Result<S> {
        let mut n = 0;
        while let Some(candidate) = self.queue.pop() {
            n += 1;

            let score = candidate.score();
            let will_process = self.process_candidate(candidate)?;

            if self.show_debug_msg(n) {
                eprintln!(
                    "Q{} R{} S{:?} ({} {}) {}",
                    self.queue.len(),
                    self.results.len(),
                    self.best().map(|p| p.score()),
                    score,
                    if will_process.is_some() { "y" } else { "n" },
                    n
                );
            }

            if let Some(candidate) = will_process {
                for child in candidate.children() {
                    self.queue.push(child);
                }
            }
            if self
                .best()
                .map(|c| self.queue.can_terminate(c))
                .unwrap_or(false)
            {
                break;
            }
        }
        self.results
            .pop()
            .map(|s| s.candidate)
            .ok_or(SearchError::NoResultFound)
    }
}
