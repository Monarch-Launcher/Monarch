#[macro_export]
/// Used to unwrap the Result type or return a sepcified value
macro_rules! unwrap_or_return {
    ( $e:expr, $return_val:expr ) => {
        match $e {
            Ok(res) => res,
            Err(_) => $return_val,
        }
    };
}

#[macro_export]
/// Used to unwrap the Option type or return a sepcified value
macro_rules! unwrap_some_or_return {
    ( $e:expr, $return_val:expr ) => {
        match $e {
            Some(res) => res,
            None => $return_val,
        }
    };
}