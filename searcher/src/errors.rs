use std::time;
use thiserror::Error;

/// Error produced when a search fails.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("No search result found")]
    NoResultFound,

    #[error("Step limit exhausted after {0} steps")]
    StepLimitExhausted(usize),

    #[error("Time limit exhausted after {0:?}")]
    TimeLimitExhausted(time::Duration),
}

/// Result when a search method might fail.
pub type Result<T> = std::result::Result<T, SearchError>;
