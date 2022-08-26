use std::{error::Error, fmt::Display};

use bytes::Bytes;


pub struct Set {
    key: String,
    value: String,
}

pub struct Get {
    key: String,
}

enum Commands {
    SET(Set),
    GET(Get),
}

#[derive(Debug, PartialEq)]
pub enum CommandError {
    DesError,
    Empty,
}

impl Error for CommandError {}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::DesError => "Deserialize error".fmt(f),
            CommandError::Empty => todo!(),
        }
    }
}
pub trait Deserialize {
    // fn start(&self) ;
    fn next_string(&mut self) -> Result<String, CommandError>;
    fn next_i64(&self) -> Result<i64, CommandError>;
}

pub trait Serialize {
    // fn start(&self) ;
    fn push_string(&mut self, val: String) -> Result<usize, CommandError>;
    fn push_i64(&mut self, val: i64) -> Result<usize, CommandError>;
    fn finish(&mut self) -> Result<Bytes, CommandError>;
}

impl Set {
    pub fn new(key: String, value: String) -> Self {
        Set {
            key,
            value,
        }
    }
}

