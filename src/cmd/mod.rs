mod get;
use bytes::Bytes;
pub use get::Get;
mod set;
pub use set::Set;

use crate::frame::{Frame, Parse};

// #[derive(Debug)]
// pub enum Error {
//     ParseError,
//     IncorrectCommand,
// }

type Error = Box<dyn std::error::Error + Send + Sync>;

/// unify the command type
/// Why?
/// because we do not know what commands it is before parsing a frame
/// after we know the command type, then we can parse the command parameters
/// parse command parameters are done by Command struct instead of this one
#[derive(Debug)]
pub enum Request {
    Get(Get),
    Set(Set),
}

impl Request {
    pub fn from_frame(frame: Frame) -> Result<Request, Error> {
        let mut it = frame.into_iterator();
        match it.next_string()?.to_lowercase() {
            cmd if cmd == "get" => Ok(Request::Get(Get::from_frame(&mut it)?)),
            cmd if cmd == "set" => Ok(Request::Set(Set::from_frame(&mut it)?)),
            _ => panic!("unknown command"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Response {
    OK,
    ERR(String),
    DATA(Bytes),
    NULL,
}


