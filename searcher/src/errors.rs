use thiserror::Error;

/// Error produced when a search fails.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("No search result found")]
    NoResultFound,

    #[error("Step limit exhausted after {0} steps")]
    StepLimitExhausted(usize),
}

pub type Result<T> = std::result::Result<T, SearchError>;
