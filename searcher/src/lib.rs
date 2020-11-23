pub mod algorithm;
mod errors;
mod traits;

pub use errors::Result as SearchResult;
pub use errors::SearchError;
pub use traits::SearchCacher;
pub use traits::SearchCandidate;
pub use traits::SearchHeuristic;

pub use algorithm::astar::{self, run as astar};
pub use algorithm::basic::bfs::{self, run as bfs};
pub use algorithm::basic::dfs::{self, run as dfs};
pub use algorithm::dijkstra::{self, run as dijkstra};
pub use algorithm::SearchOptions;
