use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonarchEgsError {
    #[error("web request failed")]
    WebRequestError(String),

    #[error("parsing object failed")]
    ParsingError(String),

    #[error("unknown error")]
    Unknown,
}
