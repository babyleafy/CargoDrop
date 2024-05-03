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

// TODO:
// There is a bug where if u connect on client side and then close the connection it breaks the server because server waits for talking

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

        if !prompt_permission(&mut lines_from_stdin).await? {
            continue 'outer;
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

            if let Err(e) = socket.write_u64(file_size).await {
                eprintln!("Failed to write to socket; err = {:?}", e);
                return;
            }

            if let Err(e) = socket.write_all(filename.as_bytes()).await {
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

async fn prompt_permission(lines_from_stdin: &mut Lines<AsyncBufReader<Stdin>>) -> Result<bool> {
    loop {
        if let Some(response) = lines_from_stdin.next_line().await? {
            match response.as_str() {
                "y" => return Ok(true),
                "n" => return Ok(false),
                _ => eprintln!("Expected [y/n]"),
            }
        }
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
