use anyhow::Result;
use std::env;

use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    const BUF_SIZ: usize = 1024;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // TODO: this is bad error handling
        panic!("Need to pass in IP address as argument for client");
    }

    let mut stream = TcpStream::connect(format!("{}:7878", args[1])).await?;
    println!("Connected to server");

    let mut buffer = [0; BUF_SIZ]; // Adjust buffer size as needed

    let file_size = stream.read_u64().await?;

    let n = stream.read(&mut buffer).await?;
    let file_name = String::from_utf8(buffer[..n].to_vec())?;

    if n == 0 {
        println!("Could not reach handshake for file acceptance");
    }
    eprintln!(
        "{} wants to send {} ({} bytes) to you. Allow file send? [y/n]",
        args[1], file_name, file_size
    );
    let mut lines_from_stdin = tokio::io::BufReader::new(io::stdin()).lines();
    loop {
        if let Some(response) = lines_from_stdin.next_line().await? {
            match response.as_str() {
                "y" => {
                    stream.write("y".as_bytes()).await?;
                    break;
                }
                "n" => {
                    return Ok(());
                }
                _ => (),
            }
        }
        eprintln!("Expected [y/n]");
    }

    let file = File::create(file_name).await?;
    let mut buffered_file = BufWriter::new(file);
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        buffered_file.write_all(&buffer[..n]).await?;
    }

    buffered_file.flush().await?;

    println!("client done");

    Ok(())
}
