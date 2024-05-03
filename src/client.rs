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

    let file = File::create("out").await?;
    let mut buffered_file = BufWriter::new(file);

    let mut buffer = [0; BUF_SIZ]; // Adjust buffer size as needed

    let n = stream.read(&mut buffer).await?;

    if n == 0 {
        println!("Could not reach handshake for file acceptance");
    }
    eprintln!("Allow file send? [y/n]");
    eprintln!("{}", String::from_utf8(buffer[..n].to_vec())?);
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
