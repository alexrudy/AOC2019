//! Generalized search algorithms, especially useful for graph traversal.
//!
//! To use these search algorithms, implement at least the trait [SearchCandidate]
//! which must provide a few methods

pub mod algorithm;
mod errors;
pub mod graph;
mod traits;

pub use algorithm::score::Score;
pub use errors::Result as SearchResult;
pub use errors::SearchError;
pub use traits::SearchCacher;
pub use traits::SearchCandidate;
pub use traits::SearchHeuristic;
pub use traits::SearchScore;
pub use traits::SearchState;

pub use algorithm::astar;
pub use algorithm::basic::bfs;
pub use algorithm::basic::dfs;
pub use algorithm::dijkstra;
pub use algorithm::SearchOptions;
