use std::io::Error;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    tokio::select! {

        _ = tokio::signal::ctrl_c() => {

            eprintln!("shutdown server");
        }
    }

    // Server::start().await;
    Ok(())
}
