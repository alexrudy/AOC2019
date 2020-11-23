use std::fmt::Debug;
use std::hash::Hash;

pub trait SearchCandidate: Debug + Sized {
    fn is_complete(&self) -> bool;

    fn score(&self) -> usize;

    fn children(&self) -> Vec<Self>;
}

pub trait SearchHeuristic: SearchCandidate {
    /// Best guess of the final score given our current distance
    fn heuristic(&self) -> usize {
        self.score()
    }
}

pub trait SearchCacher: SearchCandidate {
    type State: Debug + Eq + Hash;

    fn state(&self) -> Self::State;
}
