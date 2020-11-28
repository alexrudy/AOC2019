//! Provides the building blocks for search algorithms

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::default::Default;
use std::time;

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

#[derive(Debug, Default)]
struct TimeLimit {
    start: Option<time::Instant>,
    maximum: Option<time::Duration>,
}

impl TimeLimit {
    fn new(limit: Option<time::Duration>) -> Self {
        Self {
            start: None,
            maximum: limit,
        }
    }

    fn increment(&mut self) -> Result<()> {
        if self.start.is_none() {
            self.start = Some(time::Instant::now());
        }
        if self
            .start
            .map(|s| self.maximum.map(|m| s.elapsed() > m).unwrap_or(false))
            .unwrap_or(false)
        {
            Err(SearchError::TimeLimitExhausted(
                self.start.unwrap().elapsed(),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default)]
struct StepLimit {
    current: usize,
    maximum: Option<usize>,
}

impl StepLimit {
    fn new(limit: Option<usize>) -> Self {
        Self {
            current: 0,
            maximum: limit,
        }
    }

    fn increment(&mut self) -> Result<()> {
        self.current += 1;

        if self.maximum.map(|v| self.current >= v).unwrap_or(false) {
            Err(SearchError::StepLimitExhausted(self.current))
        } else {
            Ok(())
        }
    }
}

/// Options for the search algorithm.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct SearchOptions {
    pub limit: Option<usize>,
    pub maxtime: Option<time::Duration>,
    pub verbose: Option<usize>,
    pub exhaustive: bool,
}

impl SearchOptions {
    pub fn new() -> Self {
        Self::default()
    }
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
    counter: StepLimit,
    timer: TimeLimit,
    options: SearchOptions,
    origin: Option<S>,
}

impl<S, Q, C> SearchAlgorithm<S, Q, C>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
    C: Cache<Candidate = S>,
{
    fn new_with_options(origin: S, options: SearchOptions) -> Self {
        let counter = StepLimit::new(options.limit);
        let timer = TimeLimit::new(options.maxtime);

        let sr = SearchAlgorithm {
            cache: C::default(),
            queue: Q::default(),
            results: BinaryHeap::default(),
            counter: counter,
            timer: timer,
            options: options,
            origin: Some(origin),
        };
        sr
    }

    fn new(origin: S) -> Self {
        Self::new_with_options(origin, SearchOptions::default())
    }

    pub fn with_options(mut self, options: SearchOptions) -> Self {
        let origin = self
            .origin
            .take()
            .expect("Algorithm appears to have already started, no origin!");

        Self::new_with_options(origin, options)
    }

    fn best(&self) -> Option<&S> {
        self.results.peek().map(|s| &s.candidate)
    }

    // Should we continue searching from this candidate?
    fn process_candidate(&mut self, candidate: S) -> Result<Option<S>> {
        // Increment the step counter
        self.counter.increment()?;
        self.timer.increment()?;

        // If we found an answer, we can stop hunting now
        // and add the answer to our search results.
        if candidate.is_complete() {
            self.results.push(Score::new(candidate));
            return Ok(None);
        }

        // Scores can only increase in searches, if the best candidate
        // is better than our current guess, give up now.
        let score = candidate.score();
        if score >= self.best().map(|s| s.score()).unwrap_or(usize::MAX) {
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
        let origin = self.origin.take().unwrap();

        if let Some(c) = self.process_candidate(origin)? {
            self.queue.push(c);
        }

        while let Some(candidate) = self.queue.pop() {
            n += 1;

            let score = candidate.score();

            if self.show_debug_msg(n) {
                eprintln!(
                    "Q{} R{} S{:?} ({}) {}",
                    self.queue.len(),
                    self.results.len(),
                    self.best().map(|p| p.score()),
                    score,
                    n
                );
            }

            for child in candidate.children() {
                if let Some(c) = self.process_candidate(child)? {
                    self.queue.push(c);
                }
            }
            if !self.options.exhaustive
                && self
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
