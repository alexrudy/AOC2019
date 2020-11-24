//! Search cacheing support to eliminate already explored items.

use std::collections::HashMap;
use std::default::Default;
use std::marker::PhantomData;

use crate::errors::Result;
use crate::traits::{SearchCacher, SearchCandidate};

/// Defines the behavior required of a search cache.
pub trait Cache: Default {
    type Candidate: SearchCandidate;

    fn check(&mut self, candidate: &Self::Candidate) -> Result<bool>;
}

/// Provides no-op caching.
#[derive(Debug)]
pub struct NoCache<T>(PhantomData<T>);

impl<T> Default for NoCache<T> {
    fn default() -> Self {
        NoCache(PhantomData)
    }
}

impl<S> Cache for NoCache<S>
where
    S: SearchCandidate,
{
    type Candidate = S;

    fn check(&mut self, _candidate: &Self::Candidate) -> Result<bool> {
        Ok(true)
    }
}

/// Provides a simple hashmap cache which
/// will store every search state encountered.
#[derive(Debug)]
pub struct BasicCache<S>
where
    S: SearchCacher,
{
    cache: HashMap<S::State, usize>,
}

impl<S> Default for BasicCache<S>
where
    S: SearchCacher,
{
    fn default() -> Self {
        BasicCache {
            cache: HashMap::default(),
        }
    }
}

impl<S> Cache for BasicCache<S>
where
    S: SearchCacher,
{
    type Candidate = S;

    fn check(&mut self, candidate: &Self::Candidate) -> Result<bool> {
        let state = candidate.state();
        let score = candidate.score();
        // Check if we have already seen this state in our cache.
        // (a) For states which are not in the cache, add them.
        // (b) If the state is already in the cache, and has a lower score,
        //     we should ignore this candidate.
        // (c) For states which are already in the cache but have a higher
        //     score, mark this state as the new winner.

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
}
