use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::default::Default;
use std::fmt::Debug;
use std::hash::Hash;

mod errors {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum SearchError {
        #[error("Step limit exhausted after {0} steps")]
        StepLimitExhausted(usize),
    }

    pub type Result<T> = std::result::Result<T, SearchError>;
}

pub(crate) trait SearchCandidate: Ord + PartialOrd + Eq + Clone + Debug + Sized {
    type State: Debug + Eq + Hash;

    fn is_complete(&self) -> bool;

    fn state(&self) -> Self::State;

    fn score(&self) -> usize;

    fn children(&self) -> Vec<Self>;

    fn heuristic(&self) -> usize {
        self.score()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct SearchResult<S>
where
    S: SearchCandidate,
{
    candidate: S,
}

impl<S> SearchResult<S>
where
    S: SearchCandidate,
{
    fn new(candidate: S) -> Self {
        Self { candidate }
    }
}

impl<S> Ord for SearchResult<S>
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

impl<S> PartialOrd for SearchResult<S>
where
    S: SearchCandidate,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) trait SearchQueue {
    type Candidate;

    fn pop(&mut self) -> Option<Self::Candidate>;

    fn push(&mut self, item: Self::Candidate);

    fn len(&self) -> usize;
}

#[derive(Debug)]
pub(crate) struct BreadthQueue<S> {
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

#[derive(Debug, Default)]
pub(crate) struct DepthQueue<S> {
    queue: VecDeque<S>,
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct DjirkstraElement<S> {
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
pub(crate) struct DjirkstraQueue<S>
where
    S: SearchCandidate,
{
    queue: BinaryHeap<DjirkstraElement<S>>,
}

impl<S> Default for DjirkstraQueue<S>
where
    S: SearchCandidate,
{
    fn default() -> Self {
        DjirkstraQueue {
            queue: BinaryHeap::new(),
        }
    }
}

impl<S> SearchQueue for DjirkstraQueue<S>
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

#[derive(Debug, Eq, PartialEq)]
struct Heuristic<S>
where
    S: Eq,
{
    candidate: S,
}

impl<S> Ord for Heuristic<S>
where
    S: SearchCandidate,
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
    S: SearchCandidate,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default)]
pub(crate) struct AStarQueue<S>
where
    S: SearchCandidate,
{
    queue: BinaryHeap<Heuristic<S>>,
}

impl<S> SearchQueue for AStarQueue<S>
where
    S: SearchCandidate,
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

    fn increment(&mut self) -> errors::Result<()> {
        self.current += 1;

        if self.current >= self.maximum {
            Err(errors::SearchError::StepLimitExhausted(self.current))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default)]
struct Searcher<S, Q>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
{
    cache: HashMap<S::State, usize>,
    queue: Q,
    results: BinaryHeap<SearchResult<S>>,
    counter: Option<StepLimit>,
}

impl<S, Q> Searcher<S, Q>
where
    S: SearchCandidate,
    Q: SearchQueue<Candidate = S> + Default,
{
    fn new(origin: S) -> Self {
        let mut sr = Searcher {
            cache: HashMap::default(),
            queue: Q::default(),
            results: BinaryHeap::default(),
            counter: None,
        };
        sr.queue.push(origin);
        sr
    }

    fn set_limit(&mut self, limit: usize) {
        self.counter = Some(StepLimit::new(limit))
    }

    fn best(&self) -> Option<&S> {
        self.results.peek().map(|s| &s.candidate)
    }

    // Should we continue searching from this candidate?
    fn process_candidate(&mut self, candidate: &S) -> errors::Result<bool> {
        // Increment the step counter
        self.counter
            .as_mut()
            .map(|c| c.increment())
            .unwrap_or(Ok(()))?;

        // If we found an answer, we can stop hunting now
        // and add the answer to our search results.

        if candidate.is_complete() {
            self.results.push(SearchResult::new(candidate.clone()));
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

        if *cached_score >= score {
            // (c)
            *cached_score = score;
        } else {
            // (b)
            return Ok(false);
        }

        return Ok(true);
    }

    fn run(&mut self) -> errors::Result<Option<&S>> {
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
        Ok(self.best())
    }
}

pub(crate) fn bfs<S>(origin: S) -> errors::Result<Option<S>>
where
    S: SearchCandidate,
{
    let mut searcher: Searcher<S, BreadthQueue<S>> = Searcher::new(origin);
    Ok(searcher.run()?.cloned())
}

pub(crate) fn djirkstra<S>(origin: S, limit: Option<usize>) -> errors::Result<Option<S>>
where
    S: SearchCandidate,
{
    let mut searcher: Searcher<S, DjirkstraQueue<S>> = Searcher::new(origin);
    if let Some(l) = limit {
        searcher.set_limit(l);
    }

    Ok(searcher.run()?.cloned())
}
