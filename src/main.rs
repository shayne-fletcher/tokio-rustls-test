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
            chunk_size,
            chunk_analysis,
        } => {
            let client = FileClient::new(server)?;

            if chunk_analysis {
                run_chunk_analysis(&client, output.as_deref()).await?;
            } else if benchmark {
                run_benchmark(&client, output.as_deref(), chunk_size).await?;
            } else {
                run_transfers(&client, size, count, chunk_size).await?;
            }
        }
    }

    Ok(())
}

async fn run_transfers(client: &FileClient, size: u64, count: usize, chunk_size: usize) -> Result<()> {
    println!(
        "Running {} transfers of {} each ({}KB chunks)",
        count,
        format_size(size, DECIMAL),
        chunk_size / 1024
    );

    let mut total_duration = Duration::ZERO;
    let mut results = Vec::new();

    for i in 1..=count {
        print!("Transfer {}/{}: ", i, count);
        let result = client.send_file(size, chunk_size).await?;

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

async fn run_benchmark(client: &FileClient, output_file: Option<&str>, chunk_size: usize) -> Result<()> {
    println!("Running benchmark with various file sizes ({}KB chunks)...\n", chunk_size / 1024);

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
        let result = client.send_file(size, chunk_size).await?;
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
    let mut csv_content = String::from("file_size_bytes,duration_ms,throughput_mbps,chunk_size_bytes\n");
    for result in results {
        csv_content.push_str(&format!(
            "{},{},{:.2},{}\n",
            result.file_size,
            result.duration.as_millis(),
            result.throughput_mbps,
            result.chunk_size
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

# Convert file sizes to MB and chunk sizes to KB for better readability
df['file_size_mb'] = df['file_size_bytes'] / (1024 * 1024)
df['chunk_size_kb'] = df['chunk_size_bytes'] / 1024

# Check if multiple chunk sizes were used
unique_chunks = df['chunk_size_bytes'].unique()
multi_chunk = len(unique_chunks) > 1

# Create the plot
if multi_chunk:
    fig, axes = plt.subplots(2, 2, figsize=(15, 10))
    
    # Plot throughput vs file size, colored by chunk size
    sc1 = axes[0,0].scatter(df['file_size_mb'], df['throughput_mbps'], c=df['chunk_size_kb'], 
                           cmap='viridis', s=60, alpha=0.7)
    axes[0,0].set_xscale('log')
    axes[0,0].set_xlabel('File Size (MB)')
    axes[0,0].set_ylabel('Throughput (Mbps)')
    axes[0,0].set_title('Throughput vs File Size (colored by chunk size)')
    axes[0,0].grid(True, alpha=0.3)
    plt.colorbar(sc1, ax=axes[0,0], label='Chunk Size (KB)')
    
    # Plot throughput vs chunk size
    axes[0,1].semilogx(df['chunk_size_kb'], df['throughput_mbps'], 'g-o', linewidth=2, markersize=6)
    axes[0,1].set_xlabel('Chunk Size (KB)')
    axes[0,1].set_ylabel('Throughput (Mbps)')
    axes[0,1].set_title('Throughput vs Chunk Size')
    axes[0,1].grid(True, alpha=0.3)
    
    # Plot duration vs file size, colored by chunk size
    sc2 = axes[1,0].scatter(df['file_size_mb'], df['duration_ms'], c=df['chunk_size_kb'], 
                           cmap='viridis', s=60, alpha=0.7)
    axes[1,0].set_xscale('log')
    axes[1,0].set_yscale('log')
    axes[1,0].set_xlabel('File Size (MB)')
    axes[1,0].set_ylabel('Duration (ms)')
    axes[1,0].set_title('Duration vs File Size (colored by chunk size)')
    axes[1,0].grid(True, alpha=0.3)
    plt.colorbar(sc2, ax=axes[1,0], label='Chunk Size (KB)')
    
    # Plot efficiency (throughput per chunk size)
    df['efficiency'] = df['throughput_mbps'] / df['chunk_size_kb']
    axes[1,1].semilogx(df['file_size_mb'], df['efficiency'], 'purple', marker='d', linewidth=2, markersize=6)
    axes[1,1].set_xlabel('File Size (MB)')
    axes[1,1].set_ylabel('Throughput per KB chunk size')
    axes[1,1].set_title('Efficiency vs File Size')
    axes[1,1].grid(True, alpha=0.3)
    
else:
    fig, axes = plt.subplots(2, 1, figsize=(12, 8))
    
    # Plot throughput vs file size
    axes[0].semilogx(df['file_size_mb'], df['throughput_mbps'], 'b-o', linewidth=2, markersize=6)
    axes[0].set_xlabel('File Size (MB)')
    axes[0].set_ylabel('Throughput (Mbps)')
    axes[0].set_title(f'TLS Performance - Throughput vs File Size ({{int(unique_chunks[0]/1024)}}KB chunks)')
    axes[0].grid(True, alpha=0.3)
    
    # Plot duration vs file size
    axes[1].loglog(df['file_size_mb'], df['duration_ms'], 'r-s', linewidth=2, markersize=6)
    axes[1].set_xlabel('File Size (MB)')
    axes[1].set_ylabel('Duration (ms)')
    axes[1].set_title(f'TLS Performance - Duration vs File Size ({{int(unique_chunks[0]/1024)}}KB chunks)')
    axes[1].grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('tokio_rustls_benchmark.png', dpi=300, bbox_inches='tight')
plt.show()

# Print summary statistics
print("Benchmark Summary:")
print(f"Max throughput: {{df['throughput_mbps'].max():.2f}} Mbps")
print(f"Min throughput: {{df['throughput_mbps'].min():.2f}} Mbps")
print(f"Avg throughput: {{df['throughput_mbps'].mean():.2f}} Mbps")
print(f"File size range: {{df['file_size_mb'].min():.3f}} MB to {{df['file_size_mb'].max():.1f}} MB")
if multi_chunk:
    print(f"Chunk sizes tested: {{', '.join([str(int(x/1024)) + 'KB' for x in sorted(unique_chunks)])}}")
    print(f"Best performing chunk size: {{int(df.loc[df['throughput_mbps'].idxmax(), 'chunk_size_bytes']/1024)}}KB")
else:
    print(f"Chunk size: {{int(unique_chunks[0]/1024)}}KB")
"#,
        csv_path
    );

    fs::write(&py_path, py_content)?;

    Ok(())
}

async fn run_chunk_analysis(client: &FileClient, output_file: Option<&str>) -> Result<()> {
    println!("Running chunk size analysis across multiple file sizes...\n");
    
    let file_sizes = vec![
        1_048_576,     // 1 MB
        10_485_760,    // 10 MB
        104_857_600,   // 100 MB
    ];
    
    let chunk_sizes = vec![
        1024,    // 1 KB
        2048,    // 2 KB
        4096,    // 4 KB
        8192,    // 8 KB
        16384,   // 16 KB
        32768,   // 32 KB
        65536,   // 64 KB
        131072,  // 128 KB
        262144,  // 256 KB
    ];
    
    println!("{:<12} {:<12} {:<12} {:<12}", "File Size", "Chunk Size", "Duration", "Throughput");
    println!("{}", "-".repeat(55));
    
    let mut results = Vec::new();
    
    for file_size in &file_sizes {
        for chunk_size in &chunk_sizes {
            let result = client.send_file(*file_size, *chunk_size).await?;
            println!(
                "{:<12} {:<12} {:<12}ms {:<12.2} Mbps",
                format_size(result.file_size, DECIMAL),
                format!("{}KB", chunk_size / 1024),
                result.duration.as_millis(),
                result.throughput_mbps
            );
            results.push(result);
        }
        println!(); // Empty line between file sizes
    }
    
    if let Some(output_path) = output_file {
        write_output_files(&results, output_path)?;
        println!("Output written to {} and {}.py", output_path, output_path.trim_end_matches(".csv"));
        println!("The Python script will show detailed chunk size analysis!");
    }
    
    // Print optimal chunk sizes for each file size
    println!("Optimal chunk sizes:");
    for file_size in &file_sizes {
        let best_result = results
            .iter()
            .filter(|r| r.file_size == *file_size)
            .max_by(|a, b| a.throughput_mbps.partial_cmp(&b.throughput_mbps).unwrap())
            .unwrap();
        
        println!(
            "{}: {}KB ({:.2} Mbps)",
            format_size(*file_size, DECIMAL),
            best_result.chunk_size / 1024,
            best_result.throughput_mbps
        );
    }
    
    Ok(())
}
