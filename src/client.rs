use anyhow::Result;
use core::panic;
use std::env;

use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<()> {
    const BUF_SIZ: usize = 1024;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Need to pass in IP address as argument for client");
    }

    let mut stream = TcpStream::connect(format!("{}:7878", args[1])).await?;

    let mut buffer = [0; BUF_SIZ]; // Adjust buffer size as needed

    println!("Attempting to connect to server");

    let mut stdin = io::BufReader::new(io::stdin());

    loop {
        eprintln!("looped");

        let file_size = match stream.read_u64().await {
            Ok(file_size) => file_size,
            Err(_) => {
                eprintln!("Server rejected your connection");
                return Ok(());
            }
        };
        if file_size == 0 {
            break;
        }

        println!("Connected to server.");

        stream.read_exact(&mut buffer).await?;
        let file_name_length = match buffer.iter().position(|x| *x == b'\0') {
            None => {
                panic!("Server did not send file name correctly");
            }
            Some(idx) => idx,
        };
        let file_name_received = String::from_utf8(buffer[..file_name_length].to_vec())?;

        println!(
            "Received file: {} (size: {} bytes)",
            file_name_received, file_size
        );
        println!("Do you want to save the file? (y/n)");

        let mut confirm = String::new();
        stdin.read_line(&mut confirm).await?;
        confirm = confirm.trim().to_lowercase();

        if confirm == "y" {
            println!("Enter a filename to save the file:");
            let mut file_name = String::new();
            stdin.read_line(&mut file_name).await?;
            file_name = file_name.trim().to_string();

            let file = File::create(&file_name).await?;
            let mut buffered_file = BufWriter::new(file);

            let mut received_size = BUF_SIZ - file_name_length - 1;

            buffered_file
                .write_all(&buffer[file_name_length + 1..])
                .await?;

            while (received_size as u64) < file_size {
                let n = stream.read(&mut buffer).await?;
                buffered_file.write_all(&buffer[..n]).await?;
                received_size += n;
            }

            buffered_file.flush().await?;
            println!("File saved: {}", file_name);
        } else {
            println!("File transfer rejected.");
            stream.read_exact(&mut buffer[..file_size as usize]).await?;
        }
    }

    println!("Client done");

    Ok(())
}
