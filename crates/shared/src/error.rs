use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipstashError {
    #[error("Failed to fetch URL: {0}")]
    FetchError(String),

    #[error("Failed to parse article content: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Article not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
