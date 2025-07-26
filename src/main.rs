mod cli;
mod client;
mod server;
mod tls_utils;

use anyhow::Result;
use cli::{parse, Commands};
use client::{FileClient, TransferResult};
use humansize::{format_size, DECIMAL};
use server::FileServer;
use std::fs;
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
            output,
        } => {
            let client = FileClient::new(server)?;

            if benchmark {
                run_benchmark(&client, output.as_deref()).await?;
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
            "{} in {}ms ({:.2} Mbps)",
            format_size(result.file_size, DECIMAL),
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

async fn run_benchmark(client: &FileClient, output_file: Option<&str>) -> Result<()> {
    println!("Running benchmark with various file sizes...\n");

    let sizes = vec![
        1024,          // 1 KB
        2_048,         // 2 KB
        4_096,         // 4 KB
        8_192,         // 8 KB
        16_384,        // 16 KB
        32_768,        // 32 KB
        65_536,        // 64 KB
        131_072,       // 128 KB
        262_144,       // 256 KB
        524_288,       // 512 KB
        1_048_576,     // 1 MB
        2_097_152,     // 2 MB
        5_242_880,     // 5 MB
        10_485_760,    // 10 MB
        20_971_520,    // 20 MB
        52_428_800,    // 50 MB
        104_857_600,   // 100 MB
        209_715_200,   // 200 MB
        524_288_000,   // 500 MB
        1_073_741_824, // 1 GB
    ];

    println!("{:<12} {:<12} {:<12}", "Size", "Duration", "Throughput");
    println!("{}", "-".repeat(40));

    let mut results = Vec::new();

    for size in sizes {
        let result = client.send_file(size).await?;
        println!(
            "{:<12} {:<12}ms {:<12.2} Mbps",
            format_size(result.file_size, DECIMAL),
            result.duration.as_millis(),
            result.throughput_mbps
        );
        results.push(result);
    }

    if let Some(output_path) = output_file {
        write_output_files(&results, output_path)?;
        println!(
            "\nOutput written to {} and {}.py",
            output_path,
            output_path.trim_end_matches(".csv")
        );
    }

    Ok(())
}

fn write_output_files(results: &[TransferResult], output_path: &str) -> Result<()> {
    let csv_path = if output_path.ends_with(".csv") {
        output_path.to_string()
    } else {
        format!("{}.csv", output_path)
    };

    let py_path = format!("{}.py", csv_path.trim_end_matches(".csv"));

    // Write CSV file
    let mut csv_content = String::from("file_size_bytes,duration_ms,throughput_mbps\n");
    for result in results {
        csv_content.push_str(&format!(
            "{},{},{:.2}\n",
            result.file_size,
            result.duration.as_millis(),
            result.throughput_mbps
        ));
    }
    fs::write(&csv_path, csv_content)?;

    // Write Python plotting script
    let py_content = format!(
        r#"#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Read the benchmark data
df = pd.read_csv('{}')

# Convert file sizes to MB for better readability
df['file_size_mb'] = df['file_size_bytes'] / (1024 * 1024)

# Create the plot
plt.figure(figsize=(12, 8))

# Plot throughput vs file size
plt.subplot(2, 1, 1)
plt.semilogx(df['file_size_mb'], df['throughput_mbps'], 'b-o', linewidth=2, markersize=6)
plt.xlabel('File Size (MB)')
plt.ylabel('Throughput (Mbps)')
plt.title('TLS File Transfer Performance - Throughput vs File Size')
plt.grid(True, alpha=0.3)

# Plot duration vs file size
plt.subplot(2, 1, 2)
plt.loglog(df['file_size_mb'], df['duration_ms'], 'r-s', linewidth=2, markersize=6)
plt.xlabel('File Size (MB)')
plt.ylabel('Duration (ms)')
plt.title('TLS File Transfer Performance - Duration vs File Size')
plt.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('tokio_rustls_benchmark.png', dpi=300, bbox_inches='tight')
plt.show()

# Print summary statistics
print("Benchmark Summary:")
print(f"Max throughput: {{df['throughput_mbps'].max():.2f}} Mbps")
print(f"Min throughput: {{df['throughput_mbps'].min():.2f}} Mbps")
print(f"Avg throughput: {{df['throughput_mbps'].mean():.2f}} Mbps")
print(f"File size range: {{df['file_size_mb'].min():.3f}} MB to {{df['file_size_mb'].max():.1f}} MB")
"#,
        csv_path
    );

    fs::write(&py_path, py_content)?;

    Ok(())
}
