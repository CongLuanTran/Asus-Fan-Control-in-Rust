use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::cli::Command;

pub async fn status(socket_path: String) {
    let mut unixstream = UnixStream::connect(Path::new(&socket_path)).await.expect("Could not connect to the socket path. Ensure that the path is correct and is being listened on.");

    let payload = serde_json::to_vec(&Command::Status).unwrap();
    match unixstream.write_all(&payload).await {
        Ok(_) => {
            let mut buf: [u8; 1024] = [0u8; 1024];
            let n = unixstream.read(&mut buf).await.unwrap();
            println!("{}", String::from_utf8_lossy(&buf[..n]));
        }
        Err(e) => {
            eprintln!("Error writing to daemon; error: {e}")
        }
    }
}
