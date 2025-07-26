#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt

# Read both result sets
v0241 = pd.read_csv('v0241_results.csv')
v0262 = pd.read_csv('v0262_results.csv')

# Convert to MB
v0241['file_size_mb'] = v0241['file_size_bytes'] / (1024 * 1024)
v0262['file_size_mb'] = v0262['file_size_bytes'] / (1024 * 1024)

# Create comparison plot
plt.figure(figsize=(15, 10))

# Throughput comparison
plt.subplot(2, 2, 1)
plt.semilogx(v0241['file_size_mb'], v0241['throughput_mbps'], 'r-o', label='v0.24.1', linewidth=2, markersize=6)
plt.semilogx(v0262['file_size_mb'], v0262['throughput_mbps'], 'b-s', label='v0.26.2', linewidth=2, markersize=6)
plt.xlabel('File Size (MB)')
plt.ylabel('Throughput (Mbps)')
plt.title('Throughput Comparison: v0.24.1 vs v0.26.2')
plt.legend()
plt.grid(True, alpha=0.3)

# Performance improvement percentage
plt.subplot(2, 2, 2)
improvement = ((v0262['throughput_mbps'] - v0241['throughput_mbps']) / v0241['throughput_mbps'] * 100)
plt.semilogx(v0262['file_size_mb'], improvement, 'g-^', linewidth=2, markersize=6)
plt.xlabel('File Size (MB)')
plt.ylabel('Performance Improvement (%)')
plt.title('Performance Improvement: v0.26.2 vs v0.24.1')
plt.grid(True, alpha=0.3)
plt.axhline(y=0, color='k', linestyle='--', alpha=0.5)

# Duration comparison
plt.subplot(2, 2, 3)
plt.loglog(v0241['file_size_mb'], v0241['duration_ms'], 'r-o', label='v0.24.1', linewidth=2, markersize=6)
plt.loglog(v0262['file_size_mb'], v0262['duration_ms'], 'b-s', label='v0.26.2', linewidth=2, markersize=6)
plt.xlabel('File Size (MB)')
plt.ylabel('Duration (ms)')
plt.title('Duration Comparison: v0.24.1 vs v0.26.2')
plt.legend()
plt.grid(True, alpha=0.3)

# Summary statistics
plt.subplot(2, 2, 4)
categories = ['1MB', '10MB', '100MB', '1GB']
v0241_vals = [v0241[v0241['file_size_bytes'] == size]['throughput_mbps'].iloc[0] 
              for size in [1048576, 10485760, 104857600, 1073741824]]
v0262_vals = [v0262[v0262['file_size_bytes'] == size]['throughput_mbps'].iloc[0] 
              for size in [1048576, 10485760, 104857600, 1073741824]]

x = range(len(categories))
width = 0.35
plt.bar([i - width/2 for i in x], v0241_vals, width, label='v0.24.1', color='red', alpha=0.7)
plt.bar([i + width/2 for i in x], v0262_vals, width, label='v0.26.2', color='blue', alpha=0.7)
plt.xlabel('File Size')
plt.ylabel('Throughput (Mbps)')
plt.title('Key File Size Comparisons')
plt.xticks(x, categories)
plt.legend()
plt.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('tokio_rustls_version_comparison.png', dpi=300, bbox_inches='tight')
plt.show()

# Print detailed comparison
print("=== TOKIO-RUSTLS VERSION COMPARISON ===")
print("v0.24.1 vs v0.26.2 Performance Analysis\n")

for size in [1048576, 10485760, 104857600, 1073741824]:
    old_perf = v0241[v0241['file_size_bytes'] == size]['throughput_mbps'].iloc[0]
    new_perf = v0262[v0262['file_size_bytes'] == size]['throughput_mbps'].iloc[0]
    improvement = (new_perf - old_perf) / old_perf * 100
    
    size_label = f"{size // (1024*1024) if size >= 1024*1024 else size // 1024}{'MB' if size >= 1024*1024 else 'KB'}"
    print(f"{size_label:>6}: {old_perf:6.1f} → {new_perf:6.1f} Mbps ({improvement:+5.1f}%)")

print(f"\nOverall Summary:")
print(f"v0.24.1 avg: {v0241['throughput_mbps'].mean():.1f} Mbps")
print(f"v0.26.2 avg: {v0262['throughput_mbps'].mean():.1f} Mbps")
print(f"Overall improvement: {(v0262['throughput_mbps'].mean() - v0241['throughput_mbps'].mean()) / v0241['throughput_mbps'].mean() * 100:+.1f}%")