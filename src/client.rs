use anyhow::Result;
use std::env;

use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    const BUF_SIZ: usize = 1024;

    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        // TODO: this is bad error handling
        panic!("Need to pass in IP address as argument for client");
    }

    let mut stream = TcpStream::connect(format!("{}:7878", args[1])).await?;
    println!("Connected to server");

    let file = File::create("out").await?;
    let mut buffered_file = BufWriter::new(file);

    let mut buffer = [0; BUF_SIZ]; // Adjust buffer size as needed

    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        buffered_file.write_all(&buffer[..n]).await?;
    }

    buffered_file.flush().await?;

    println!("client done");

    // let n = stream.read(&mut buffer).await?;
    // println!(
    //     "Received response from server: {}",
    //     String::from_utf8_lossy(&buffer[..n])
    // );

    Ok(())
}
