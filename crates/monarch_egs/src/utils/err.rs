use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MonarchEgsError {
    #[error("web request failed")]
    WebRequestError(String),

    #[error("parsing object failed")]
    ParsingError(String),

    #[error("hash mismatch")]
    HashMismatchError(String),

    #[error("download cancelled")]
    Cancelled,

    #[error("unknown error")]
    Unknown,
}
