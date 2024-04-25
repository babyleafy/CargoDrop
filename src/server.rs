use tokio::net::TcpListener;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:7878").await?;
    println!("Server running on port 7878");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buf = vec![0; 4096];  // Adjust buffer size as needed

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed by client");
                        return;
                    }
                    Ok(n) => {
                        println!("Received {} bytes: {}", n, String::from_utf8_lossy(&buf[..n]));
                        if let Err(e) = socket.write_all(&buf[..n]).await {
                            eprintln!("Failed to write to socket; err = {:?}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read from socket; err = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
