# Benchmarking tokio-rustls Versions

This document describes how to benchmark and compare different versions of tokio-rustls.

## Version Comparison Procedure

### 1. Setup Baseline (Current Version)

On the main branch, run a benchmark to establish the baseline:

```bash
git checkout master
cargo run -- client --benchmark --output v0262_results
```

This generates `v0262_results.csv` with performance data for the current version.

### 2. Setup Comparison Version

Create a branch for testing an older version:

```bash
git checkout -b test-tokio-rustls-0.24.1
```

Edit `Cargo.toml` to downgrade dependencies:

```toml
[dependencies]
tokio-rustls = "0.24.1"
rustls = { version = "0.21", features = ["dangerous_configuration"] }
rustls-pemfile = "1.0"
# ... other deps
```

Update code for API compatibility (see git history for specific changes needed between versions).

### 3. Run Comparison Benchmark

```bash
cargo run -- client --benchmark --output v0241_results
```

This generates `v0241_results.csv` with performance data for the comparison version.

### 4. Generate Analysis

Switch back to master and run the analysis:

```bash
git checkout master
python version_comparison.py
```

This creates:
- `tokio_rustls_version_comparison.png` - Performance visualization
- Console output with detailed statistics

## Benchmark Parameters

Each benchmark tests 20 file sizes from 1KB to 1GB:
- Chunk size: 8192 bytes
- Single transfer per size
- Metrics: throughput (Mbps), duration (ms)

## Adding New Versions

To compare additional versions:

1. Create a new branch: `git checkout -b test-tokio-rustls-X.Y.Z`
2. Update `Cargo.toml` with target version
3. Fix any API compatibility issues
4. Run: `cargo run -- client --benchmark --output vXYZ_results`
5. Update `version_comparison.py` to include the new dataset
6. Generate new comparison plots

## Results Format

CSV files contain:
- `file_size_bytes`: Size of transferred file
- `duration_ms`: Transfer time in milliseconds  
- `throughput_mbps`: Calculated throughput in Mbps
- `chunk_size_bytes`: Buffer size used for transfer (when present)