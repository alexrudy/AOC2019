use std::cmp::{Ord, PartialOrd};
use std::fmt::Debug;
use std::hash::Hash;

pub trait SearchCandidate: Ord + PartialOrd + Eq + Clone + Debug + Sized {
    type State: Debug + Eq + Hash;

    fn is_complete(&self) -> bool;

    fn state(&self) -> Self::State;

    fn score(&self) -> usize;

    fn children(&self) -> Vec<Self>;
}

pub trait SearchHeuristic: SearchCandidate {
    /// Best guess of the final score given our current distance
    fn heuristic(&self) -> usize {
        self.score()
    }
}
