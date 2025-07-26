use crate::tls_utils::create_server_config;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

pub struct FileServer {
    acceptor: TlsAcceptor,
    addr: SocketAddr,
}

impl FileServer {
    pub fn new(cert_pem: &str, key_pem: &str, addr: SocketAddr) -> Result<Self> {
        let acceptor = create_server_config(cert_pem, key_pem)?;
        Ok(Self { acceptor, addr })
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Server listening on {}", self.addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = self.acceptor.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(acceptor, stream, peer_addr).await {
                    eprintln!("Error handling connection from {}: {}", peer_addr, e);
                }
            });
        }
    }
}

async fn handle_connection(
    acceptor: TlsAcceptor,
    stream: TcpStream,
    peer_addr: SocketAddr,
) -> Result<()> {
    let mut tls_stream = acceptor.accept(stream).await?;
    
    let mut size_bytes = [0u8; 8];
    tls_stream.read_exact(&mut size_bytes).await?;
    let file_size = u64::from_le_bytes(size_bytes);
    
    println!("Receiving file of {} bytes from {}", file_size, peer_addr);
    
    let mut received = 0u64;
    let mut buffer = vec![0u8; 8192];
    
    while received < file_size {
        let bytes_to_read = std::cmp::min(buffer.len(), (file_size - received) as usize);
        let bytes_read = tls_stream.read(&mut buffer[..bytes_to_read]).await?;
        
        if bytes_read == 0 {
            return Err(anyhow::anyhow!("Connection closed before receiving all data"));
        }
        
        received += bytes_read as u64;
    }
    
    tls_stream.write_all(b"OK").await?;
    println!("Successfully received {} bytes from {}", received, peer_addr);
    
    Ok(())
}