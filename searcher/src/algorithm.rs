//! Provides the building blocks for search algorithms

use std::collections::BinaryHeap;
use std::default::Default;
use std::time;

use self::cache::Cache;
use self::score::Score;
use crate::errors::{Result, SearchError};
use crate::traits::{SearchCandidate, SearchScore};

pub mod astar;
pub mod basic;
pub mod cache;
pub mod dijkstra;
pub mod score;

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
    S: SearchCandidate + Ord,
    Q: SearchQueue<Candidate = S> + Default,
    C: Cache<Candidate = S>,
{
    cache: C,
    queue: Q,
    results: BinaryHeap<S>,
    counter: StepLimit,
    timer: TimeLimit,
    options: SearchOptions,
    origin: Option<S>,
}

impl<S, Q, C> SearchAlgorithm<Score<S>, Q, C>
where
    S: SearchScore,
    Q: SearchQueue<Candidate = Score<S>> + Default,
    C: Cache<Candidate = Score<S>>,
{
    pub fn new_with_score(origin: S) -> Self {
        Self::new(Score::from(origin))
    }
}

impl<S, Q, C> SearchAlgorithm<S, Q, C>
where
    S: SearchCandidate + Ord,
    Q: SearchQueue<Candidate = S> + Default,
    C: Cache<Candidate = S>,
{
    pub fn new_with_options(origin: S, options: SearchOptions) -> Self {
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

    pub fn new(origin: S) -> Self {
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
        self.results.peek()
    }

    // Should we continue searching from this candidate?
    fn process_candidate(&mut self, candidate: S) -> Result<Option<S>> {
        // Increment the step counter
        self.counter.increment()?;
        self.timer.increment()?;

        // If we found an answer, we can stop hunting now
        // and add the answer to our search results.
        if candidate.is_complete() {
            self.results.push(candidate);
            return Ok(None);
        }

        // Scores can only increase in searches, if the best candidate
        // is better than our current guess, give up now.
        if self.best().map(|s| &candidate >= s).unwrap_or(false) {
            return Ok(None);
        }

        if self.cache.check(&candidate)? {
            return Ok(Some(candidate));
        }
        Ok(None)
    }

    fn show_debug_msg(&self, n: usize) -> bool {
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

            if self.show_debug_msg(n) {
                eprintln!("Q{} R{} {}", self.queue.len(), self.results.len(), n);
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
        self.results.pop().ok_or(SearchError::NoResultFound)
    }
}
