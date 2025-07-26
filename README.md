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
cargo run -- client --benchmark
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

## Example Output

```
Running benchmark with various file sizes...

Size         Duration     Throughput  
----------------------------------------
1.0 KB       15ms         0.53 Mbps
10.0 KB      12ms         6.67 Mbps
100.0 KB     18ms         44.44 Mbps
1.0 MB       45ms         177.78 Mbps
10.0 MB      89ms         897.65 Mbps
100.0 MB     890ms        898.88 Mbps
```

## Build

```bash
cargo build --release
```

## Dependencies

- tokio + tokio-rustls for async TLS
- rcgen for certificate generation
- clap for CLI parsing
- anyhow for error handling