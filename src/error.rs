use std::io::{Error, ErrorKind};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn new_error(kind: &str) -> Box<Error> {
    Box::new(Error::new(ErrorKind::Other, kind))
}

