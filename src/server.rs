use std::sync::Arc;
use anyhow::Result;
use local_ip_address::local_ip;
use tokio::fs;
use tokio::fs::File as AsyncFile;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader, Lines, Stdin};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up ip and a listener on the ip port
    let my_local_ip = local_ip()?;
    let listener = TcpListener::bind(format!("{:?}:7878", my_local_ip)).await?;
    println!("Server running at {:?} on port 7878", my_local_ip);

    // Track connections
    let connections = Arc::new(Mutex::new(Vec::new()));
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn task to listen for incoming files to send to all clients
    tokio::spawn({
        let connections = connections.clone();
        async move {
            while let Some(filename) = rx.recv().await {
                broadcast_file_to_all(&filename, &connections).await;
            }
        }
    });

    let mut stdin_lines = tokio::io::BufReader::new(io::stdin()).lines();
    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                let (socket, addr) = accept_result?;
                eprintln!("New connection from {}. Allow connection? [y/n]", addr);

                if prompt_permission(&mut stdin_lines).await? {
                    let mut connections_lock = connections.lock().await;
                    connections_lock.push(socket);
                    eprintln!("Connection from {} accepted.", addr);
                } else {
                    println!("Connection from {} rejected.", addr);
                }
            },
            Ok(line) = stdin_lines.next_line() => {
                process_command_input(line, &tx).await;
            },
        }
    }
}

async fn broadcast_file_to_all(filename: &String, connections: &Arc<Mutex<Vec<TcpStream>>>) {
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

    if let Err(e) = socket.write_all(filename).await {
        eprintln!("Failed to write to socket; err = {:?}", e);
        return;
    }

    if let Err(e) = socket.write_u8(filename).await {
        eprintln!("Failed to write to socket; err = {:?}", e);
        return;
    }

    let mut connections_lock = connections.lock().await;

    for socket in connections_lock.iter_mut() {
        /*if let Err(e) = socket.write_u64(file_size).await {
            eprintln!("Failed to write to socket; err = {:?}", e);
            continue;
        }

        if let Err(e) = socket.write_all(filename.as_bytes()).await {
            eprintln!("Failed to write to socket; err = {:?}", e);
            continue;
        }*/

        let mut buf = vec![0; 4096]; // Adjust buffer size as needed

        let mut reader = AsyncBufReader::new(file.try_clone().await.unwrap());

        loop {
            match reader.read(&mut buf).await {
                Ok(0) => {
                    println!("Finished sending file to {}", socket.peer_addr().unwrap());
                    break;
                }
                Ok(n) => {
                    if let Err(e) = socket.write_all(&buf[..n]).await {
                        eprintln!("Failed to write to socket; err = {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from file; err = {:?}", e);
                    break;
                }
            }
        }
    }
}

async fn process_command_input(line: Option<String>, tx: &mpsc::Sender<String>) {
    if let Some(command) = line.expect("CLI command").trim().strip_prefix("send ") {
        if !command.is_empty() {
            tx.send(command.to_string()).await.unwrap();
        } else {
            println!("Error: Missing filename after 'send'.");
        }
    } else {
        println!("Unrecognized command or missing filename. Expected format: 'send [filename]'");
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
