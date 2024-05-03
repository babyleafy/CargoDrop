use anyhow::Result;
use local_ip_address::local_ip;
use tokio::fs;
use tokio::fs::File as AsyncFile;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader, Lines, Stdin};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

//TODO:
// 1. Make this more shell like supporting commands like ls and showing who is connected
// 2. Make server choose which client can connect

#[tokio::main]
async fn main() -> Result<()> {
    // Set up ip and a listener on the ip port
    let my_local_ip = local_ip()?;
    let listener = TcpListener::bind(format!("{:?}:7878", my_local_ip)).await?;
    println!("Server running at {:?} on port 7878", my_local_ip);

    let mut lines_from_stdin = tokio::io::BufReader::new(io::stdin()).lines();
    'outer: loop {
        let (mut socket, addr) = listener.accept().await?;
        eprintln!("New connection from {}. Allow connection? [y/n]", addr);

        loop {
            if let Some(response) = lines_from_stdin.next_line().await? {
                match response.as_str() {
                    "y" => {
                        break;
                    }
                    "n" => {
                        // TODO: send some sort of response telling client to close connection
                        // Actually, socket goes out of scope so it automatically does this
                        continue 'outer;
                    }
                    _ => (),
                }
            }
            eprintln!("Expected [y/n]");
        }

        eprintln!("Enter path of file to send:");
        let filename = next_line_from_stdin(&mut lines_from_stdin).await?;

        tokio::spawn(async move {
            let file = match AsyncFile::open(&filename).await {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open file: {:?}", e);
                    return;
                }
            };

            let file_size = match fs::metadata(&filename).await {
                Ok(file_metadata) => file_metadata.len(),
                Err(e) => {
                    eprintln!("Failed to open file: {:?}", e);
                    return;
                }
            };

            let msg_to_client = format!(
                "{} wants to send {} ({} bytes) to you. Accept? [y/n]",
                my_local_ip, &filename, file_size
            );

            if let Err(e) = socket.write_all(msg_to_client.as_bytes()).await {
                eprintln!("Failed to write to socket; err = {:?}", e);
                return;
            }

            let mut buf = vec![0; 4096]; // Adjust buffer size as needed

            match socket.read(&mut buf).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to read response from client; err = {:?}", e);
                    return;
                }
            };

            println!("hi");

            let mut reader = AsyncBufReader::new(file);

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

// Gets the next line from stdin
async fn next_line_from_stdin(stdin_lines: &mut Lines<AsyncBufReader<Stdin>>) -> Result<String> {
    loop {
        if let Some(response) = stdin_lines.next_line().await? {
            return Ok(response);
        }
    }
}
