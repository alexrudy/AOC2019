pub use bfs::bfs;
pub use dfs::dfs;

mod bfs {
    use std::collections::VecDeque;
    use std::default::Default;

    use crate::algorithm::cache::NoCache;
    use crate::algorithm::{SearchAlgorithm, SearchQueue};
    use crate::errors::Result;
    use crate::SearchCandidate;

    #[derive(Debug)]
    pub struct BreadthQueue<S> {
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
    type BreadthFirstSearcher<S> = SearchAlgorithm<S, BreadthQueue<S>, NoCache<S>>;

    fn build<S>(origin: S) -> BreadthFirstSearcher<S>
    where
        S: SearchCandidate,
    {
        SearchAlgorithm::new(origin)
    }

    /// Breadth-first search, where the order is determined
    /// by the candidates returned by the [SearchCandidate::children] method.
    pub fn bfs<S>(origin: S) -> Result<S>
    where
        S: SearchCandidate,
    {
        build(origin).run()
    }
}

mod dfs {
    use std::collections::VecDeque;
    use std::default::Default;

    use crate::algorithm::cache::NoCache;
    use crate::algorithm::{SearchAlgorithm, SearchQueue};
    use crate::errors::Result;
    use crate::SearchCandidate;
    #[derive(Debug)]
    pub struct DepthQueue<S> {
        queue: VecDeque<S>,
    }

    impl<S> Default for DepthQueue<S> {
        fn default() -> Self {
            DepthQueue {
                queue: VecDeque::new(),
            }
        }
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

    pub type DepthFirstSearcher<S> = SearchAlgorithm<S, DepthQueue<S>, NoCache<S>>;
    pub fn build<S>(origin: S) -> DepthFirstSearcher<S>
    where
        S: SearchCandidate,
    {
        SearchAlgorithm::new(origin)
    }

    /// Depth-first search, where the order is determined
    /// by the candidates returned by the [SearchCandidate::children] method.
    pub fn dfs<S>(origin: S) -> Result<S>
    where
        S: SearchCandidate,
    {
        build(origin).run()
    }
}
