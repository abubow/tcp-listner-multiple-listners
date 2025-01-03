use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = echo(stream).await {
                        eprintln!("Failed to process connection; error = {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection; error = {}", e);
            }
        }
    }
}

async fn echo(mut stream: tokio::net::TcpStream) -> io::Result<()> {
    let mut buf = [0; 1024];

    loop {
        let n = match stream.read(&mut buf).await {
            Ok(0) => return Ok(()), // Connection closed by client
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                return Err(e);
            }
        };

        println!("Read {} bytes: {:?}", n, &buf[..n]);

        if let Err(e) = stream.write_all(&buf[..n]).await {
            eprintln!("Failed to write to socket; err = {:?}", e);
            return Err(e);
        }
    }
}
