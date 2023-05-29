#[derive(Debug)]
pub enum MonarchResult<T, E> {
    Ok(T),
    MonarchErr(E),
}

#[derive(Debug)]
pub enum MonarchErr {
    FunctionErr(String),
    ConnectionErr(String),
    IOErr(String),
    UnknownErr(String),
}