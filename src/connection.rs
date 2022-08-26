use std::io::Cursor;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::{self, Runtime},
    time::{sleep_until, Duration, Instant, Sleep},
};

use crate::{command::Deserialize, frame::Frame};

/// network layer
///
pub struct Connection {
    stream: TcpStream,
    write_buffer: BytesMut,
    read_buffer: BytesMut,
}

#[derive(Debug)]
pub enum Error {
    WriteError(String),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Self {
        Error::WriteError(src.to_string())
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
            if let Ok(_) = Frame::check(&mut cursor) {
                cursor.set_position(0);
                let frame = Frame::decode(&mut cursor).unwrap();
                let len = cursor.position() as usize;
                self.read_buffer.advance(len);
                return Ok(frame);
            }
        }

        loop {
            let len = self.stream.read_buf(&mut self.read_buffer).await.unwrap();
            println!("read: {}", len);

            let mut cursor = Cursor::new(self.read_buffer.as_ref());
            match Frame::check(&mut cursor) {
                Ok(_) => {
                    cursor.set_position(0);
                    let frame = Frame::decode(&mut cursor).unwrap();
                    let len = cursor.position() as usize;
                    self.read_buffer.advance(len);
                    return Ok(frame);
                }
                Err(_) => continue,
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.stream.shutdown();
    }
}

#[test]
fn test_connection() {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        const LOOPS: usize = 10000;

        for i in 0..LOOPS {
            let mut frame = Frame::array();
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
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        const LOOPS: usize = 10000;
        for i in 0..LOOPS {
            let mut frame = Frame::array();
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
