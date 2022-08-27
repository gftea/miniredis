use bytes::Bytes;
use miniredis::{
    cmd,
    connection::{Connection},
    frame::{Frame}, database::Database,
};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime,
};

#[test]
fn test_set_cmd() {
    new_runtime().block_on(async {
        const server_addr: &str = "127.0.0.1:6381";
        start_server(server_addr).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let stream = TcpStream::connect(server_addr).await.unwrap();
        let mut conn = Connection::new(stream).unwrap();

        let cmd = cmd::Set::new("name".to_string(), Bytes::from("simon"));
        let frame = cmd.into_frame();
        let len = conn.write_frame(frame).await.unwrap();
        // println!("written: {}", len);

        let ans = conn.read_frame().await.unwrap();
        assert_eq!(ans, "OK");
        // println!("{i}");


        let cmd = cmd::Get::new("name".to_string());
        let frame = cmd.into_frame();
        let len = conn.write_frame(frame).await.unwrap();
        // println!("written: {}", len);

        let ans = conn.read_frame().await.unwrap();
        assert_eq!(ans, "simon");
    });
}



#[test]
fn test_get_cmd() {
    new_runtime().block_on(async {
        const server_addr: &str = "127.0.0.1:6380";
        start_server(server_addr).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let stream = TcpStream::connect(server_addr).await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        const LOOPS: usize = 2;

        for i in 0..LOOPS {
            let cmd = cmd::Get::new("name".to_string());
            let frame = cmd.into_frame();
            let len = conn.write_frame(frame).await.unwrap();
            // println!("written: {}", len);

            let ans = conn.read_frame().await.unwrap();
            assert_eq!(ans, Frame::Null);
            // println!("{i}");
        }
    });
}
#[test]
fn test_get_cmd_external_server() {
    new_runtime().block_on(async {
        let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        const LOOPS: usize = 2;

        for i in 0..LOOPS {
            let cmd = cmd::Get::new("name".to_string());
            let frame = cmd.into_frame();
            let len = conn.write_frame(frame).await.unwrap();
            // println!("written: {}", len);

            let ans = conn.read_frame().await.unwrap();
            assert_eq!(ans, Frame::Null);
            // println!("{i}");
        }
    });
}
async fn start_server(addr: &'static str) {
    let mut db = Database::new();
    tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut conn = Connection::new(stream).unwrap();
        println!("accept connection");
        loop {
            let frame = conn.read_frame().await.unwrap();
            let req = cmd::Request::from_frame(frame).unwrap();
            println!("receive {:?}", req);

            match req {
                cmd::Request::Get(cmd) => cmd.apply(&db, &mut conn).await.unwrap(),
                cmd::Request::Set(cmd) => cmd.apply(&mut db, &mut conn).await.unwrap()
            };
        }
    });
}
fn new_runtime() -> runtime::Runtime {
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt
}
