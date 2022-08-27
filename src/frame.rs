use core::panic;
use std::{
    fmt,
    io::{BufWriter, Cursor},
    num::TryFromIntError,
    str::Utf8Error,
    string::FromUtf8Error,
    vec,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::io::AsyncWriteExt;

#[derive(Debug, PartialEq)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

const FLAG_SIMPLE: u8 = b'+';
const FLAG_ERROR: u8 = b'-';
const FLAG_INTEGER: u8 = b':';
const FLAG_BULK: u8 = b'$';
const FLAG_ARRAY: u8 = b'*';
const FLAG_NULL: u8 = b'-';
const CRLF: &[u8] = b"\r\n";
const NULL_BULK: &[u8] = b"-1\r\n";

#[derive(Debug)]
pub enum Error {
    Incomplete,

    Other(String),
}
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(_) => todo!(),
        }
    }
}

fn get_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !buf.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(buf.get_u8())
}
fn peek_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !buf.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(buf.chunk()[0])
}
fn get_line<'a>(buf: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = buf.position() as usize;
    let end = buf.get_ref().len() - 1;
    for i in start..end {
        if buf.get_ref()[i] == b'\r' && buf.get_ref()[i + 1] == b'\n' {
            buf.set_position((i + 2) as u64);
            return Ok(&buf.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}

/// Read a new-line terminated decimal
fn get_decimal(buf: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(buf)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

fn skip(buf: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if buf.remaining() < n {
        return Err(Error::Incomplete);
    }

    buf.advance(n);
    Ok(())
}

impl Frame {
    // check if `buf` has complete frame
    pub fn check(buf: &mut Cursor<&[u8]>) -> Result<(), Error> {
        let flag = get_u8(buf)?;
        match flag {
            FLAG_SIMPLE => {
                get_line(buf)?;
                Ok(())
            }
            FLAG_ERROR => {
                get_line(buf)?;
                Ok(())
            }
            FLAG_INTEGER => {
                get_line(buf)?;
                Ok(())
            }
            FLAG_BULK => {
                if FLAG_NULL == peek_u8(buf)? {
                    skip(buf, 4)
                } else {
                    let len: usize = get_decimal(buf)?.try_into()?;
                    skip(buf, len + 2)
                }
            }
            FLAG_ARRAY => {
                if FLAG_NULL == peek_u8(buf)? {
                    skip(buf, 4)
                } else {
                    let len = get_decimal(buf)?;

                    for _ in 0..len {
                        Frame::check(buf)?;
                    }
                    Ok(())
                }
            }
            unknown => Err(format!("protocol error; invalid frame type byte `{}`", unknown).into()),
        }
    }

    // parse the frame from `buf`
    pub fn decode(buf: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(buf)? {
            FLAG_SIMPLE => {
                let s = get_line(buf)?;
                Ok(Frame::Simple(std::str::from_utf8(s)?.to_string()))
            }
            FLAG_ERROR => {
                let s = get_line(buf)?;
                Ok(Frame::Error(std::str::from_utf8(s)?.to_string()))
            }
            FLAG_INTEGER => {
                let n = get_decimal(buf)?;

                Ok(Frame::Integer(n))
            }
            FLAG_BULK => {
                if FLAG_NULL == peek_u8(buf)? {
                    match get_line(buf)? {
                        b"-1" => Ok(Frame::Null),
                        unknown => Err(format!(
                            "protocol error: invalid bulk frame format {:?}",
                            unknown
                        )
                        .into()),
                    }
                } else {
                    let len = get_decimal(buf)? as usize;
                    let end = len + 2;
                    if buf.remaining() < end {
                        return Err(Error::Incomplete);
                    }
                    //
                    let data: Bytes = Bytes::copy_from_slice(&buf.chunk()[..len]);
                    skip(buf, end)?;
                    Ok(Frame::Bulk(data))
                }
            }
            FLAG_ARRAY => {
                if FLAG_NULL == peek_u8(buf)? {
                    match get_line(buf)? {
                        b"-1" => Ok(Frame::Null),
                        unknown => Err(format!(
                            "protocol error: invalid array frame format {:?}",
                            unknown
                        )
                        .into()),
                    }
                } else {
                    let len = get_decimal(buf)?.try_into()?;
                    let mut data = Vec::with_capacity(len);
                    for _ in 0..len {
                        data.push(Frame::decode(buf)?);
                    }
                    Ok(Frame::Array(data))
                }
            }
            unknown => Err(format!("protocol error; invalid frame type byte `{}`", unknown).into()),
        }
    }

    pub fn encode<T: BufMut>(&self, buf: &mut T) -> usize {
        match self {
            Frame::Simple(s) => {
                buf.put_u8(FLAG_SIMPLE);
                buf.put(s.as_bytes());
                buf.put(CRLF);
                1 + s.len() + CRLF.len()
            }
            Frame::Error(s) => {
                buf.put_u8(FLAG_ERROR);
                buf.put(s.as_bytes());
                buf.put(CRLF);
                1 + s.len() + CRLF.len()
            }
            Frame::Integer(n) => {
                buf.put_u8(FLAG_INTEGER);
                let n = n.to_string();
                buf.put(n.as_bytes());
                buf.put(CRLF);
                1 + n.to_string().len() + CRLF.len()
            }
            Frame::Bulk(bs) => {
                buf.put_u8(FLAG_BULK);
                let n = bs.len().to_string();
                buf.put(n.as_bytes());
                buf.put(CRLF);
                buf.put(bs.as_ref());
                buf.put(CRLF);
                1 + n.len() + CRLF.len() + bs.len() + CRLF.len()
            }
            Frame::Null => {
                buf.put_u8(FLAG_BULK);
                buf.put(NULL_BULK);
                1 + NULL_BULK.len()
            }
            Frame::Array(arr) => {
                let mut written = 0;
                buf.put_u8(FLAG_ARRAY);
                let n = arr.len().to_string();
                buf.put(n.as_bytes());
                buf.put(CRLF);
                written += 1 + n.len() + CRLF.len();
                for frame in arr {
                    written += frame.encode(buf);
                }
                written
            }
        }
    }
    /// Returns an empty array
    pub(crate) fn new_array_frame() -> Frame {
        Frame::Array(vec![])
    }

    /// Push a "bulk" frame into the array. `self` must be an Array frame.
    ///
    /// # Panics
    ///
    /// panics if `self` is not an array
    pub(crate) fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Bulk(bytes));
            }
            _ => panic!("not an array frame"),
        }
    }

    /// Push an "integer" frame into the array. `self` must be an Array frame.
    ///
    /// # Panics
    ///
    /// panics if `self` is not an array
    pub(crate) fn push_int(&mut self, value: u64) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Integer(value));
            }
            _ => panic!("not an array frame"),
        }
    }

    pub(crate) fn into_iterator(self) -> vec::IntoIter<Frame> {
        match self {
            Frame::Array(vec) => {
                vec.into_iter()
            }
            _ => panic!("must be array frame")
        }
    }
}


/// This trait better to be in command module
/// why?
/// it means this is the interfaces required by command module, 
/// - user put requirements on interfaces
/// - provider implement the interface
/// 
pub(crate) trait Parse {
    fn next_string(&mut self) -> Result<String, Error>;
    fn next_bytes(&mut self) -> Result<Bytes, Error>;
    fn next_int(&mut self) -> Result<u64, Error>;
}


impl Parse for vec::IntoIter<Frame> {
    fn next_string(self: &mut std::vec::IntoIter<Frame>) -> Result<String, Error>  {
        let frame = self.next().ok_or(Error::Other("not a string".to_string()))?;
        match frame {
            Frame::Bulk(bs) => {
                Ok(String::from_utf8(bs.as_ref().to_vec())?)
            },
            Frame::Simple(s) => {
                Ok(s)
            },
            _ => Err(Error::Other("not a frame with string".to_string()))
        }
    }

    fn next_bytes(&mut self) -> Result<Bytes, Error> {
        let frame = self.next().ok_or(Error::Other("not a bytes".to_string()))?;
        match frame {
            Frame::Bulk(bs) => {
                Ok(bs)
            },
            _ => Err(Error::Other("not a frame with bytes".to_string()))
        }
    }

    fn next_int(&mut self) -> Result<u64, Error> {
        let frame = self.next().ok_or(Error::Other("not a bytes".to_string()))?;
        match frame {
            Frame::Integer(n) => {
                Ok(n)
            }
            _ => Err(Error::Other("not a integer frame".to_string()))
        }
    }
}


impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}
impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(src: FromUtf8Error) -> Self {
        src.to_string().into()
    }
}
impl From<Utf8Error> for Error {
    fn from(src: Utf8Error) -> Self {
        src.to_string().into()
    }
}
impl From<TryFromIntError> for Error {
    fn from(src: TryFromIntError) -> Self {
        src.to_string().into()
    }
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(s) => s.eq(other),
            Frame::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}


//////////////////////////////
/// Unit Test
////////////////////////////// 
#[test]
#[should_panic(expected = "protocol error")]
fn test_check() {
    let mut buf = Cursor::new(&b"123\r\n"[..]);
    Frame::check(&mut buf).unwrap()
}

#[test]
fn test_decode() {
    let mut buf = Cursor::new(&b"$5\r\nHello\r\n"[..]);
    assert_eq!(Frame::check(&mut buf).unwrap(), ());
    buf.set_position(0);
    assert_eq!(Frame::decode(&mut buf).unwrap(), "Hello");
}
#[test]
fn test_encode() {
    let frame = Frame::Bulk("Hello".into());

    let buf = &mut vec![];
    frame.encode(buf);
    assert_eq!(b"$5\r\nHello\r\n".to_vec(), *buf);

    let mut buf = [0u8; 100];
    let n = frame.encode(&mut &mut buf[..]);
    assert_eq!(b"$5\r\nHello\r\n".to_vec(), buf[..n]);

    let frame = Frame::Array(vec![
        Frame::Bulk("set".into()),
        Frame::Bulk("Hello".into()),
        Frame::Bulk("world".into()),
    ]);

    let mut buf = BytesMut::with_capacity(100);
    frame.encode(&mut buf);
    assert_eq!(
        b"*3\r\n$3\r\nset\r\n$5\r\nHello\r\n$5\r\nworld\r\n".to_vec(),
        buf
    );
}
