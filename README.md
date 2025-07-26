# tokio-rustls-test

A Rust CLI tool for testing file transfer performance using tokio-rustls.

## Features

- **TLS Server**: Accepts secure connections and receives file transfers
- **TLS Client**: Connects to server and sends test files of varying sizes
- **Benchmark Mode**: Tests throughput across multiple file sizes (1KB to 100MB)
- **Self-signed Certificates**: Automatically generates certificates for testing
- **Performance Metrics**: Reports transfer duration and throughput in Mbps

## Usage

### Start the Server

```bash
cargo run -- server --addr 127.0.0.1:8443
```

### Run Benchmark

```bash
# Basic benchmark (console output)
cargo run -- client --benchmark

# Benchmark with data export and plotting
cargo run -- client --benchmark --output results
```

### Custom Tests

```bash
# Single 10MB transfer
cargo run -- client --size 10485760

# Multiple transfers
cargo run -- client --size 1048576 --count 5

# Custom server address
cargo run -- client --server 192.168.1.100:8443 --benchmark
```

### Data Analysis

When using `--output`, the tool generates:
- `results.csv` - Raw benchmark data
- `results.py` - Python matplotlib script for visualization

```bash
# Install plotting dependencies
conda install pandas matplotlib

# Generate plots
python results.py
```

## Example Output

```
Running benchmark with various file sizes...

Size         Duration     Throughput  
----------------------------------------
1.02 kB      9ms          0.89 Mbps
4.10 kB      2ms          12.78 Mbps
64.00 kB     2ms          211.63 Mbps
1.05 MB      3ms          2405.73 Mbps
10.49 MB     15ms         5468.63 Mbps
104.86 MB    145ms        5760.30 Mbps
1.07 GB      1334ms       6435.98 Mbps
```

The benchmark now tests 20 different file sizes from 1KB to 1GB, providing detailed performance curves for analysis.

## Build

```bash
cargo build --release
```

## Dependencies

- tokio + tokio-rustls for async TLS
- rcgen for certificate generation
- clap for CLI parsing
- anyhow for error handling