pub mod algorithm;
mod errors;
mod traits;

pub use errors::Result as SearchResult;
pub use errors::SearchError;
pub use traits::SearchCandidate;
pub use traits::SearchHeuristic;

pub use algorithm::astar::{astar, AStarSearcher};
pub use algorithm::basic::{bfs, BreadthFirstSearcher};
pub use algorithm::basic::{dfs, DepthFirstSearcher};
pub use algorithm::dijkstra::{djirkstra, DijkstraSearch};
