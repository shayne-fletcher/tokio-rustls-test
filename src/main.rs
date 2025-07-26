mod cli;
mod client;
mod server;
mod tls_utils;

use anyhow::Result;
use cli::{parse, Commands};
use client::FileClient;
use humansize::{format_size, DECIMAL};
use server::FileServer;
use std::time::Duration;
use tls_utils::generate_self_signed_cert;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = parse();

    match cli.command {
        Commands::Server { addr } => {
            println!("Generating self-signed certificate...");
            let (cert_pem, key_pem) = generate_self_signed_cert()?;

            let server = FileServer::new(&cert_pem, &key_pem, addr)?;
            server.start().await?;
        }
        Commands::Client {
            server,
            size,
            count,
            benchmark,
        } => {
            let client = FileClient::new(server)?;

            if benchmark {
                run_benchmark(&client).await?;
            } else {
                run_transfers(&client, size, count).await?;
            }
        }
    }

    Ok(())
}

async fn run_transfers(client: &FileClient, size: u64, count: usize) -> Result<()> {
    println!(
        "Running {} transfers of {} each",
        count,
        format_size(size, DECIMAL)
    );

    let mut total_duration = Duration::ZERO;
    let mut results = Vec::new();

    for i in 1..=count {
        print!("Transfer {}/{}: ", i, count);
        let result = client.send_file(size).await?;

        println!(
            "{}ms ({:.2} Mbps)",
            result.duration.as_millis(),
            result.throughput_mbps
        );

        total_duration += result.duration;
        results.push(result);
    }

    if count > 1 {
        let avg_duration = total_duration / count as u32;
        let avg_throughput: f64 =
            results.iter().map(|r| r.throughput_mbps).sum::<f64>() / count as f64;

        println!("\nSummary:");
        println!("Average duration: {}ms", avg_duration.as_millis());
        println!("Average throughput: {:.2} Mbps", avg_throughput);
    }

    Ok(())
}

async fn run_benchmark(client: &FileClient) -> Result<()> {
    println!("Running benchmark with various file sizes...\n");

    let sizes = vec![
        1024,        // 1 KB
        10_240,      // 10 KB
        102_400,     // 100 KB
        1_048_576,   // 1 MB
        10_485_760,  // 10 MB
        104_857_600, // 100 MB
    ];

    println!("{:<12} {:<12} {:<12}", "Size", "Duration", "Throughput");
    println!("{}", "-".repeat(40));

    for size in sizes {
        let result = client.send_file(size).await?;
        println!(
            "{:<12} {:<12}ms {:<12.2} Mbps",
            format_size(size, DECIMAL),
            result.duration.as_millis(),
            result.throughput_mbps
        );
    }

    Ok(())
}
