use anyhow::Result;
use std::env;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        // TODO: this is bad error handling
        panic!("Need to pass in IP address as argument for client");
    }

    let mut stream = TcpStream::connect(format!("{}:7878", args[1])).await?;
    println!("Connected to server");

    let msg = b"Hello, server!";
    stream.write_all(msg).await?;
    println!("Sent message to server: {}", String::from_utf8_lossy(msg));

    let mut buffer = [0; 1024]; // Adjust buffer size as needed
    let n = stream.read(&mut buffer).await?;
    println!(
        "Received response from server: {}",
        String::from_utf8_lossy(&buffer[..n])
    );

    Ok(())
}
