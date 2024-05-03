use std::fmt::Error;
use std::path::Path;
use anyhow::Result;
use local_ip_address::local_ip;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

//TODO:
// 1. Make this more shell like supporting commands like ls and showing who is connected
// 2. Make server choose which client can connect

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(100);

    let my_local_ip = local_ip()?;
    let listener = TcpListener::bind(format!("{:?}:7878", my_local_ip)).await?;
    println!("Server running at {:?} on port 7878", my_local_ip);

    tokio::spawn(async move {
        let mut lines = io::BufReader::new(io::stdin()).lines();
        loop {
            print!("server> ");
            io::stdout().flush().await.expect("TODO: panic message");
            if let Ok(line) = lines.next_line().await {
                let _ = tx.send(line).await;
            }
        }
    });

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        if let Some(Some(filename)) = rx.recv().await {
            tokio::spawn(async move {
                let file = match AsyncFile::open(filename).await {
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
}

fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<_> = input.trim().split_whitespace().collect();
    match parts.get(0) {
        Some(&"file-out") if parts.len() == 2 => Some(Command::FileOut(parts[1].to_string())),
        _ => None,
    }
}

enum Command {
    FileOut(String),
}