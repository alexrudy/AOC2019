use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{BinaryHeap, HashMap};
use std::default::Default;

use crate::errors::{Result, SearchError};
use crate::traits::SearchCandidate;

pub(crate) mod astar;
pub(crate) mod basic;
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
pub struct SearchAlgorithm<S, Q>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
{
    cache: HashMap<S::State, usize>,
    queue: Q,
    results: BinaryHeap<Score<S>>,
    counter: Option<StepLimit>,
}

impl<S, Q> SearchAlgorithm<S, Q>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
{
    fn new(origin: S) -> Self {
        let mut sr = SearchAlgorithm {
            cache: HashMap::default(),
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

        // Check if we have already seen this state in our cache.
        // (a) For states which are not in the cache, add them.
        // (b) If the state is already in the cache, and has a lower score,
        //     we should ignore this candidate.
        // (c) For states which are already in the cache but have a higher
        //     score, mark this state as the new winner.

        let state = candidate.state();

        // (a)
        let cached_score = self.cache.entry(state).or_insert(usize::MAX);

        if *cached_score > score {
            // (c)
            *cached_score = score;
        } else {
            // (b)
            return Ok(false);
        }

        return Ok(true);
    }

    pub fn run(mut self) -> Result<S> {
        let mut n = 0;
        while let Some(candidate) = self.queue.pop() {
            n += 1;

            let will_process = self.process_candidate(&candidate)?;
            if n % 10_000 == 0 {
                eprintln!(
                    "Q{} C{} R{} S{:?} ({:?} {} {}) {}",
                    self.queue.len(),
                    self.cache.len(),
                    self.results.len(),
                    self.best().map(|p| p.score()),
                    candidate.state(),
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
