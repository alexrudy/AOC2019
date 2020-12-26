use std::fmt::Debug;
use std::hash::Hash;

/// Provides an interface for conducting searches.
///
/// Searches are "complete" when they are ready to be
/// returned to the user. The complete search found with
/// the lowest score is the one which is returned to the
/// user by the search algorithm.
pub trait SearchCandidate: Debug + Sized {
    /// Indicates that this candidate should be considered
    /// complete, and causes the search algorithm to stop
    /// considering children of this candidate.
    fn is_complete(&self) -> bool;

    /// Produces additional candidates to examine in this
    /// search. Candidates need not be complete.
    fn children(&self) -> Vec<Self>;
}

pub trait SearchScore: SearchCandidate {
    type Score: Debug + Clone + PartialOrd + Ord;

    /// Determines how to rank otherwise identical candidates.
    /// In the search result, the candidate with the highest
    /// score will be returned.
    fn score(&self) -> Self::Score;
}

/// An interface for searching when a heuristic can be provided.
///
/// For incomplete searchers, the heuristic should be the best
/// guess at the minimum score achievable given the current
/// position in the search.
pub trait SearchHeuristic: SearchCandidate {
    type Hueristic: Debug + PartialOrd + Ord;

    /// Best guess of the final score given our current score.
    fn heuristic(&self) -> Self::Hueristic;
}

pub trait SearchState: SearchCandidate + Clone {
    type State: Debug + Eq + Hash;
    fn state(&self) -> Self::State;
}

/// An interface for search objects which can be cached.
///
/// Some search alrgorithms require that a state be recorded
/// to prevent backtracking or to identify the next best
/// candidate. States will be stored along with scores at a
/// given state. When checking a new candidate, if it produces
/// an existing state but with a higher or equal score to the
/// state already observed, the new candidate will be skipped.
pub trait SearchCacher: SearchState {
    type Value: Debug + Ord + Clone;

    fn value(&self) -> Self::Value;
}
