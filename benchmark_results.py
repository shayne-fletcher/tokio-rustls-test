#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Read the benchmark data
df = pd.read_csv('benchmark_results.csv')

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
print(f"Max throughput: {df['throughput_mbps'].max():.2f} Mbps")
print(f"Min throughput: {df['throughput_mbps'].min():.2f} Mbps")
print(f"Avg throughput: {df['throughput_mbps'].mean():.2f} Mbps")
print(f"File size range: {df['file_size_mb'].min():.3f} MB to {df['file_size_mb'].max():.1f} MB")
