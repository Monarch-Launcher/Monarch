#[derive(Debug)]
pub enum MonarchResult<T, E> {
    Ok(T),
    MonarchErr(E),
}

#[derive(Debug)]
pub enum MonarchErr {
    FunctionErr,
    ConnectionErr,
    IOErr,
    UnknownErr,
}