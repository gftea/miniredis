use std::{io::Error};

use miniredis::server::{start};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    println!("start server");
    start("127.0.0.1:6379").await?;
    Ok(())
}
