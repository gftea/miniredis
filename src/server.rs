use std::io;

use crate::{cmd::Request, connection::Connection, database::Database, frame::Frame};
use tokio::{net::TcpListener, sync::broadcast};
pub struct Server {
    // shared database
    db: Database,

    // shutdown notice
    shutdown_broacaster: broadcast::Sender<()>,
}

pub struct Handler {
    connection: Connection,
    shutdown_receiver: broadcast::Receiver<()>,
}

impl Handler {
    pub async fn start(&mut self) {
        println!("start hanlder");
        let mut peer_shutdown = false;
        loop {
            let frame = tokio::select! {
                res = self.connection.read_frame(), if !peer_shutdown => {
                    match res {
                        Ok(frame) => frame,
                        Err(msg) => {println!("{msg:?}"); peer_shutdown = true; continue;}
                    }
                }
                _ = self.shutdown_receiver.recv() => {
                    println!("shutdown received for connection");
                    return;
                }
            };
            println!("receive a frame");

            // let frame = self.connection.read_frame().await.unwrap();
            let req = Request::from_frame(frame).unwrap();
            match req {
                Request::Get(_) => {
                    self.connection
                        .write_frame(Frame::Simple("simon".to_string()))
                        .await
                        .unwrap();
                }
                Request::Set(_) => {
                    self.connection
                        .write_frame(Frame::Simple("OK".to_string()))
                        .await
                        .unwrap();
                }
            }
        }
    }
}

impl Server {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1);
        Server {
            db: Database::new(),
            shutdown_broacaster: tx,
        }
    }


}
pub async fn start( addr: &str) -> Result<(), io::Error> {
    let listener = TcpListener::bind(addr).await?;
    let (stream, _) = listener.accept().await?;
    let connection = Connection::new(stream).unwrap();
    let server = Server::new();
    let rx = server.shutdown_broacaster.subscribe();
    let mut handler = Handler {
        connection,
        shutdown_receiver: rx,
    };

    let join = tokio::spawn(async move {
        handler.start().await;
    });
    tokio::select! {
        Err(_) = join => {
            println!("handler return with error")
        }
        _ = tokio::signal::ctrl_c() => {
            drop(server);
            // server.shutdown_broacaster.send(());
            use std::thread;
            use std::time::Duration;
            thread::sleep(Duration::from_millis(10));
            println!("shutdown server");
        }
    };
    
    Ok(())
}