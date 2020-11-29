use std::cmp::{Ord, Ordering, PartialOrd};
use std::convert::From;

use crate::traits::{SearchCacher, SearchCandidate, SearchScore, SearchState};

#[derive(Debug, Clone)]
pub struct Score<S>
where
    S: SearchCandidate + SearchScore,
{
    score: <S as SearchScore>::Score,
    candidate: S,
}

impl<S> Score<S>
where
    S: SearchCandidate + SearchScore,
{
    fn new(candidate: S) -> Self {
        Self {
            score: candidate.score(),
            candidate,
        }
    }

    pub fn unwrap(self) -> S {
        self.candidate
    }
}

impl<S> From<S> for Score<S>
where
    S: SearchCandidate + SearchScore,
{
    fn from(candidate: S) -> Self {
        Score::new(candidate)
    }
}

impl<S> SearchCandidate for Score<S>
where
    S: SearchCandidate + SearchScore,
{
    fn is_complete(&self) -> bool {
        self.candidate.is_complete()
    }

    fn children(&self) -> Vec<Self> {
        self.candidate
            .children()
            .into_iter()
            .map(|c| Score::new(c))
            .collect()
    }
}

impl<S> SearchState for Score<S>
where
    S: SearchCandidate + SearchState + SearchScore,
{
    type State = S::State;

    fn state(&self) -> Self::State {
        self.candidate.state()
    }
}

impl<S> SearchCacher for Score<S>
where
    S: SearchCandidate + SearchState + SearchScore,
{
    type Value = S::Score;

    fn value(&self) -> Self::Value {
        self.score.clone()
    }
}

impl<S> PartialEq for Score<S>
where
    S: SearchCandidate + SearchScore,
{
    fn eq(&self, other: &Self) -> bool {
        self.score.eq(&other.score)
    }
}

impl<S> Eq for Score<S> where S: SearchCandidate + SearchScore {}

impl<S> Ord for Score<S>
where
    S: SearchCandidate + SearchScore,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score).reverse()
    }
}

impl<S> PartialOrd for Score<S>
where
    S: SearchCandidate + SearchScore,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
