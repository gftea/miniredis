use crate::{frame::{Error, Frame, Parse}, connection::Connection, database::Database};
use bytes::Bytes;

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl Get {
    pub fn new(key: String) -> Self {
        Get { key }
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::new_array_frame();
        frame.push_bulk(Bytes::from("get"));
        frame.push_bulk(Bytes::from(self.key));
        frame
    }

    /// only visiable in `cmd` module
    /// TODO: currently it returns frame::Error, should use command Error
    pub(super) fn from_frame(it: &mut impl Parse) -> Result<Self, Error> {
        Ok(Get::new(it.next_string()?))
    }

    pub async fn apply(&self, db: &Database, conn: &mut Connection) -> Result<(), super::Error> {
        // generic behavior handling without knowing underlying storage and connection?
        /// interfaces for send command, receive command and get data
        /// 
        /// 
        match db.get(&self.key) {
            Some(bs) => conn.write_frame(Frame::Bulk(bs)).await?,
            None => conn.write_frame(Frame::Null).await?,
        };
        Ok(())
        
    }
}
