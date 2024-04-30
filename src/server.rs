use anyhow::Result;
use local_ip_address::local_ip;
use std::env;
use tokio::fs::File as AsyncFile;
use tokio::io::BufReader as AsyncBufReader;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("No filename provided"));
    }
    let filename = &args[1];

    let my_local_ip = local_ip()?;
    let listener = TcpListener::bind(format!("{:?}:7878", my_local_ip)).await?;
    println!("Server running at {:?} on port 7878", my_local_ip);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        let filename = filename.clone();
        tokio::spawn(async move {
            let file = match AsyncFile::open(&filename).await {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open file: {:?}", e);
                    return;
                }
            };

            let mut reader = AsyncBufReader::new(file);
            let mut buf = vec![0; 4096]; // Adjust buffer size as needed

            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => {
                        println!("Finished sending file");
                        return;
                    }
                    Ok(n) => {
                        if let Err(e) = socket.write_all(&buf[..n]).await {
                            eprintln!("Failed to write to socket; err = {:?}", e);
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from file; err = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
