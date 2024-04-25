use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    println!("Connected to server");

    let msg = b"Hello, server!";
    stream.write_all(msg).await?;
    println!("Sent message to server: {}", String::from_utf8_lossy(msg));

    let mut buffer = [0; 1024];  // Adjust buffer size as needed
    let n = stream.read(&mut buffer).await?;
    println!("Received response from server: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}
