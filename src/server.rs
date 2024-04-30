use anyhow::Result;
use local_ip_address::local_ip;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let my_local_ip = local_ip()?;

    let listener = TcpListener::bind(format!("{:?}:7878", my_local_ip)).await?;
    println!("Server running at {:?} on port 7878", my_local_ip);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buf = vec![0; 4096]; // Adjust buffer size as needed

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed by client");
                        return;
                    }
                    Ok(n) => {
                        println!(
                            "Received {} bytes: {}",
                            n,
                            String::from_utf8_lossy(&buf[..n])
                        );
                        if let Err(e) = socket.write_all(&buf[..n]).await {
                            eprintln!("Failed to write to socket; err = {:?}", e);
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket; err = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
