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
    cache: HashMap<S::State, S::Value>,
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
    S: SearchCacher + Ord + PartialOrd,
{
    type Candidate = S;

    fn check(&mut self, candidate: &Self::Candidate) -> Result<bool> {
        let state = candidate.state();
        // Check if we have already seen this state in our cache.
        // (a) For states which are not in the cache, add them.
        // (b) If the state is already in the cache, and has a lower score,
        //     we should ignore this candidate.
        // (c) For states which are already in the cache but have a higher
        //     score, mark this state as the new winner.

        let mut r = true;
        let value = candidate.value();
        // (a)
        self.cache
            .entry(state)
            .and_modify(|e| {
                if *e > value {
                    *e = value.clone();
                } else {
                    r = false;
                }
            })
            .or_insert_with(|| value.clone());

        return Ok(r);
    }
}
