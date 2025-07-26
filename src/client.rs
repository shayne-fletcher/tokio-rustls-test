use crate::tls_utils::create_client_config;
use anyhow::Result;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

#[derive(Clone)]
pub struct TransferResult {
    pub file_size: u64,
    pub duration: Duration,
    pub throughput_mbps: f64,
}

impl TransferResult {
    pub fn new(file_size: u64, duration: Duration) -> Self {
        let throughput_mbps = (file_size as f64 * 8.0) / (duration.as_secs_f64() * 1_000_000.0);
        Self {
            file_size,
            duration,
            throughput_mbps,
        }
    }
}

pub struct FileClient {
    connector: TlsConnector,
    server_addr: SocketAddr,
}

impl FileClient {
    pub fn new(server_addr: SocketAddr) -> Result<Self> {
        let connector = create_client_config()?;
        Ok(Self {
            connector,
            server_addr,
        })
    }

    pub async fn send_file(&self, file_size: u64) -> Result<TransferResult> {
        let start_time = Instant::now();
        
        let stream = TcpStream::connect(self.server_addr).await?;
        let domain = rustls::pki_types::ServerName::try_from("localhost")?;
        let mut tls_stream = self.connector.connect(domain, stream).await?;
        
        tls_stream.write_all(&file_size.to_le_bytes()).await?;
        
        let mut sent = 0u64;
        let chunk_size = 8192;
        let data_chunk = vec![0u8; chunk_size];
        
        while sent < file_size {
            let bytes_to_send = std::cmp::min(chunk_size, (file_size - sent) as usize);
            tls_stream.write_all(&data_chunk[..bytes_to_send]).await?;
            sent += bytes_to_send as u64;
        }
        
        tls_stream.flush().await?;
        
        let mut response = [0u8; 2];
        tls_stream.read_exact(&mut response).await?;
        
        if &response != b"OK" {
            return Err(anyhow::anyhow!("Server did not acknowledge transfer"));
        }
        
        tls_stream.shutdown().await?;
        
        let duration = start_time.elapsed();
        Ok(TransferResult::new(file_size, duration))
    }
}