use std::{io::Cursor};

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::frame::{Frame, self};

/// network layer
///
pub struct Connection {
    stream: TcpStream,
    write_buffer: BytesMut,
    read_buffer: BytesMut,
}

#[derive(Debug)]
pub enum Error {
    IO(String),
    Other(String),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(msg) => write!(f, "Connection IO error: {}", msg),
            Error::Other(err) => write!(f, "Connection other error: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Self {
        Error::IO(src.to_string())
    }
}
impl From<frame::Error> for Error {
    fn from(src: frame::Error) -> Self {
        Error::Other(src.to_string())
    }
}
impl Connection {
    pub fn new(stream: TcpStream) -> Result<Connection, Error> {
        Ok(Connection {
            stream,
            write_buffer: BytesMut::with_capacity(1024),
            read_buffer: BytesMut::with_capacity(1024),
        })
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<usize, Error> {
        let written = frame.encode(&mut self.write_buffer);
        self.stream.write_all(self.write_buffer.as_ref()).await?;
        self.write_buffer.advance(written);
        Ok(written)
    }

    pub async fn read_frame(&mut self) -> Result<Frame, Error> {
        if self.read_buffer.has_remaining() {
            let mut cursor = Cursor::new(self.read_buffer.as_ref());
            if Frame::check(&mut cursor).is_ok() {
                cursor.set_position(0);
                let frame = Frame::decode(&mut cursor)?;
                let len = cursor.position() as usize;
                self.read_buffer.advance(len);
                return Ok(frame);
            }
        }

        loop {
            let len = self.stream.read_buf(&mut self.read_buffer).await?;
            if len == 0 {
                if self.read_buffer.is_empty() {
                    return Err(Error::Other("peer shutdown".to_string()));
                } else {
                    return Err(Error::IO("connection failure".to_string()));
                }
            }
            println!("read: {}", len);

            let mut cursor = Cursor::new(self.read_buffer.as_ref());
            match Frame::check(&mut cursor) {
                Ok(_) => {
                    cursor.set_position(0);
                    let frame = Frame::decode(&mut cursor)?;
                    let len = cursor.position() as usize;
                    self.read_buffer.advance(len);
                    return Ok(frame);
                }
                Err(_) => continue,
            }
        }
    }
}


//////////////////////////////
/// Unit Test
////////////////////////////// 
use tokio::runtime;

fn new_runtime() -> runtime::Runtime {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt
}
#[test]
fn test_connection() {
    new_runtime().block_on(async {
        let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        const LOOPS: usize = 10000;

        for i in 0..LOOPS {
            let mut frame = Frame::new_array_frame();
            frame.push_bulk("get".into());
            frame.push_bulk("name".into());
            let len = conn.write_frame(frame).await.unwrap();
            // println!("written: {}", len);

            let ans = conn.read_frame().await.unwrap();
            assert_eq!(ans, "simon");
            // println!("{i}");
        }
    });
}
#[test]
fn test_pipeline() {
    new_runtime().block_on(async {
        let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        const LOOPS: usize = 10000;
        for i in 0..LOOPS {
            let mut frame = Frame::new_array_frame();
            frame.push_bulk("get".into());
            frame.push_bulk("name".into());
            let len = conn.write_frame(frame).await.unwrap();
            println!("written: {}", len);

            // println!("{i}");
        }
        for i in 0..LOOPS {
            let ans = conn.read_frame().await.unwrap();
            assert_eq!(ans, "simon");
            // println!("{i}");
        }
    });
}
