use bytes::Bytes;

use crate::{frame::{Error, Frame, Parse}, database::Database, connection::Connection};
#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
}

impl Set {
    pub fn new(key: &str, value: Bytes) -> Self {
        Set { key: key.to_string(), value }
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::new_array_frame();
        frame.push_bulk(Bytes::from("set"));
        frame.push_bulk(Bytes::from(self.key));
        frame.push_bulk(self.value);

        frame
    }

    pub(super) fn from_frame(it: &mut dyn Parse) -> Result<Self, Error> {
        let key = it.next_string()?;
        let value = it.next_bytes()?;
        Ok(Set::new(&key, value))
    }
    pub async fn apply(&self, db: &mut Database, conn: &mut Connection) -> Result<(), super::Error> {
        // generic behavior handling without knowing underlying storage and connection?
        /// interfaces for send command, receive command and get data
        /// 
        /// 
        match db.set(self.key.clone(), self.value.clone()) {
            // exisiting key
            Some(_bs) => conn.write_frame(Frame::Simple("OK".to_string())).await?,
            // new key
            None => conn.write_frame(Frame::Simple("OK".to_string())).await?,
        };
        Ok(())
        
    }
}
