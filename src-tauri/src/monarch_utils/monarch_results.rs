#[derive(Debug)]
pub enum MonarchResult<T, E> {
    Ok(T),
    MonarchErr(E),
}

#[derive(Debug)]
pub enum MonarchErr {
    RequestErr(String),
    FunctionErr(String),
    IOErr(String),
    SystemErr(String),
    UnknownErr(String),
}