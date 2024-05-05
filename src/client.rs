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

    let mut buffer = [0; BUF_SIZ]; // Adjust buffer size as needed

    println!("Connected to server");

    stream.flush().await?;

    let mut stdin = io::BufReader::new(io::stdin());
    let mut file_name = String::new();

    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        println!("Received data from the server. Do you want to save the file? (y/n)");
        let mut confirm = String::new();
        stdin.read_line(&mut confirm).await?;
        confirm = confirm.trim().to_lowercase();

        if confirm == "y" {
            println!("Enter a file name to save the data:");
            stdin.read_line(&mut file_name).await?;
            file_name = file_name.trim().to_string();

            let file = File::create(&file_name).await?;
            let mut buffered_file = BufWriter::new(file);
            buffered_file.write_all(&buffer[..n]).await?;
            buffered_file.flush().await?;

            println!("Data saved to file: {}", file_name);
        } else {
            println!("File transfer rejected.");
        }

        file_name.clear();
    }

    println!("Client done");

    Ok(())
}