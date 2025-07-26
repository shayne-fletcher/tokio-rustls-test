use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "tokio-rustls-test")]
#[command(about = "A CLI tool for testing file transfer performance with tokio-rustls")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Server {
        #[arg(short, long, default_value = "127.0.0.1:8443")]
        addr: SocketAddr,
    },
    Client {
        #[arg(short = 's', long, default_value = "127.0.0.1:8443")]
        server: SocketAddr,

        #[arg(long, default_value = "1048576")]
        size: u64,

        #[arg(short = 'c', long, default_value = "1")]
        count: usize,

        #[arg(long)]
        benchmark: bool,

        #[arg(long)]
        output: Option<String>,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
