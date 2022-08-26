use crate::{request::Request, response::Response};

pub struct Client;

#[derive(Debug)]
pub enum Error {

}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Client {
    pub async fn connect() -> Result<Client, Error> {
        Ok(Client)
    }

    pub async fn send_request(&self, request: Request) -> Result<Response, Error> {
        
        Ok(Response)
    }

    pub async fn disconnect(&self, ) -> Result<(), Error> {
        Ok(())
    }
}
