use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MonarchEgsError {
    #[error("web request failed: {0}")]
    WebRequestError(String),

    #[error("parsing object failed: {0}")]
    ParsingError(String),

    #[error("hash mismatch: {0}")]
    HashMismatchError(String),

    #[error("download cancelled")]
    Cancelled,

    #[error("unknown error")]
    Unknown,
}
