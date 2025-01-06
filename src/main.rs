use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use std::io;
use pseudoterminal::{CommandExt, Terminal};
use std::io::{stdin, stdout, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
// use std::process::Command;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("WebSocket server listening on 127.0.0.1:8080");

    // Use Arc<Mutex<Vec<Uuid>>> for shared and mutable access
    let running_processes: Arc<Mutex<Vec<Uuid>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("New connection established");
                let running_processes = Arc::clone(&running_processes);


                tokio::spawn(async move {
                    let id = Uuid::new_v4();
                    let mut binding = Command::new("sh");
                    let mut process = binding.
                                            stdin(Stdio::piped())
                                            .stdout(Stdio::piped())
                                            .stderr(Stdio::piped());
                    let mut terminal = process.spawn_terminal().unwrap();
                    {
                        // Lock the Mutex to modify the shared data
                        let mut processes = running_processes.lock().unwrap();
                        processes.push(id);
                    }
                    if let Err(e) = handle_connection(stream, id, terminal).await {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        };

        // Clone and lock the Arc to access the number of active connections
        let running_processes = Arc::clone(&running_processes);
        let active_connections = {
            let processes = running_processes.lock().unwrap();
            processes.len()
        };
        println!("Active connections: {}", active_connections);
    }
}
async fn handle_connection(stream: tokio::net::TcpStream, id: Uuid,mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    println!("New WebSocket connection established: {}", id);

    let (mut write, mut read) = ws_stream.split();

    // Send a welcome message to the client
    let message = Message::text("id: ".to_string() + &id.to_string());
    write.send(message).await?;
    let mut output_buffer = [0u8; 1024];

    // run reader thread
    tokio::spawn(async move {
        while let Ok(n) = terminal.termout.as_mut().unwrap().read(&mut output_buffer) {
            if n > 0 {
                let message = Message::Text(String::from_utf8(output_buffer[0..n].to_vec()).unwrap().into());
                print!("{}", message);
                write.send(message).await.unwrap();
            }
        }
    });

    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {
                if message.is_text() || message.is_binary() {
                    // println!("Received message: {} from {}", message, id);
                    let msg = message.to_string() + "\n";
                    terminal
                        .termin
                        .as_mut()
                        .unwrap()
                        .write_all(msg.to_string().as_bytes()).unwrap();
                } else if message.is_close() {
                    println!("Client disconnected: {}", id);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading message from {}: {}", id, e);
                break;
            }
        }
    }

    Ok(())
}
