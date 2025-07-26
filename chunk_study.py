#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Read the benchmark data
df = pd.read_csv('chunk_study.csv')

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
    axes[0].set_title(f'TLS Performance - Throughput vs File Size ({int(unique_chunks[0]/1024)}KB chunks)')
    axes[0].grid(True, alpha=0.3)
    
    # Plot duration vs file size
    axes[1].loglog(df['file_size_mb'], df['duration_ms'], 'r-s', linewidth=2, markersize=6)
    axes[1].set_xlabel('File Size (MB)')
    axes[1].set_ylabel('Duration (ms)')
    axes[1].set_title(f'TLS Performance - Duration vs File Size ({int(unique_chunks[0]/1024)}KB chunks)')
    axes[1].grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('tokio_rustls_benchmark.png', dpi=300, bbox_inches='tight')
plt.show()

# Print summary statistics
print("Benchmark Summary:")
print(f"Max throughput: {df['throughput_mbps'].max():.2f} Mbps")
print(f"Min throughput: {df['throughput_mbps'].min():.2f} Mbps")
print(f"Avg throughput: {df['throughput_mbps'].mean():.2f} Mbps")
print(f"File size range: {df['file_size_mb'].min():.3f} MB to {df['file_size_mb'].max():.1f} MB")
if multi_chunk:
    print(f"Chunk sizes tested: {', '.join([str(int(x/1024)) + 'KB' for x in sorted(unique_chunks)])}")
    print(f"Best performing chunk size: {int(df.loc[df['throughput_mbps'].idxmax(), 'chunk_size_bytes']/1024)}KB")
else:
    print(f"Chunk size: {int(unique_chunks[0]/1024)}KB")
