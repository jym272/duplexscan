# `DuplexScan`

---

DuplexScan is a high-performance contact deduplication tool written in Rust that helps identify potential duplicate contacts in large datasets.
It uses advanced string similarity algorithms and parallel processing to efficiently detect and
score similar contact records on a scale from **0 to 1000**.

## Features

- 🚀 Fast parallel processing using Rayon
- 🌐 Unicode-aware string similarity comparison
- 🎚️ Configurable similarity threshold filtering
- 📊 CSV input/output support
- 🧠 Memory-efficient processing for large datasets
- 🖥️ CLI interface with flexible options

## Installation

DuplexScan provides three different ways to build/install the binary, each with its own advantages and trade-offs:

| Build Method   | Performance | Portability                | Dependencies     | Best For                |
| -------------- | ----------- | -------------------------- | ---------------- | ----------------------- |
| Local Build    | ⭐⭐⭐ Best | Limited to similar systems | System libraries | Development, Local use  |
| Docker (glibc) | ⭐⭐⭐ Best | Most Linux systems         | Requires glibc   | Production, General use |
| Docker (musl)  | ⭐ Slower   | Any Linux system           | None             | Minimal environments    |

### 1. Local Build (Best Performance)

Best for local development and when building for identical systems:

```bash
git clone https://github.com/jym272/duplexscan.git
cd duplexscan
cargo build --release
./target/release/duplexscan -f input.csv -o output.csv
```

- ✅ Fastest performance (optimized for your CPU)
- ✅ Perfect for development
- ❌ Less portable across different systems

### 2. Docker Build with glibc (Recommended)

Best balance of performance and portability:

```bash
# Using provided glibc.Dockerfile
make duplexscan-glibc
./duplexscan -f input.csv -o output.csv
```

- ✅ Near-native performance
- ✅ Works on most Linux distributions
- ❌ Requires glibc on target system

### 3. Docker Build with musl (Most Portable)

Best for maximum compatibility and containerized environments:

```bash
# Using provided build.Dockerfile
make duplexscan
./duplexscan -f input.csv -o output.csv
```

- ✅ Runs anywhere on Linux (including Alpine)
- ✅ No external dependencies
- ✅ Perfect for minimal containers
- ❌ Significantly slower performance

### Future Cross-Platform Support

We plan to add support for Windows and macOS through:

- Cross-compilation with multiple targets
- Platform-specific build pipelines
- GitHub Actions with native runners

Choose your installation method based on your specific needs:

- For development: Use local build
- For general deployment: Use glibc Docker build
- For maximum compatibility: Use musl Docker build

## Usage

Basic usage, default threshold **(800)**:

```bash
./target/release/duplexscan -f input.csv -o output.csv
# or
./duplexscan -f input.csv -o output.csv
```

With similarity threshold **(0-1000)**:

```bash
./duplexscan -f input.csv -o output.csv -t 700
```

### Command Line Options

- `-f, --file <PATH>` : Input CSV file path (required)
- `-o, --output <PATH>` : Output CSV file path (required)
- `-t, --threshold <NUMBER>` : Minimum similarity threshold (0-1000, default: 800)
- `-h, --help` : Prints help information

### Input Format

The input csv file should have the following columns:

| contactID | name | name1 | email        | postalZip | address |
| --------- | ---- | ----- | ------------ | --------- | ------- |
| 1001      | John | Doe   | john@doe.com | 12345     | 123 St  |

### Output Format

The output csv file will contain:

| ContactID1 | ContactID2 | SimilarityScore |
| ---------- | ---------- | --------------- |
| 1001       | 1002       | 850             |

## How It Works

1. **Contact Loading**: Reads contacts from the input csv file
2. **Similarity Calculation**:
   - Compares each contact pair using:
     - First Name (weight: 2)
     - Last Name (weight: 2)
     - Email (weight: 3)
     - Zip Code (weight: 2)
     - Address (weight: 2)
   - Uses Levenshtein distance for string comparisons
3. **Score Normalization**:
   - Scores are normalized to a 0-1000 scale
   - Higher scores indicate greater similarity
4. **Threshold Filtering**:
   - Optional filtering of results based on minimum similarity score
   - Reduces output size for large datasets

## Performance

### Complexity Analysis

1. **Overall Time Complexity: O(N²L)** where:

   - N is the number of contacts
   - L is the maximum length of any field in the contacts

   The quadratic complexity comes from comparing every contact with every other contact. Each comparison involves string similarity calculations that depend on the length of the fields.

2. **Overall Space Complexity: O(N² + RC)** where:
   - N² comes from storing all pair combinations for similarity scores
   - R is the number of rows in the input file
   - C is the average size of contact data

### Key bottlenecks and optimizations:

1. The comparison of all contact pairs (O(N²)) is parallelized using Rayon
2. String comparisons use efficient Unicode-aware Levenshtein distance
3. Memory usage is optimized by using references where possible, Rust's ownership system ensures memory safety without garbage collection
4. File I/O is handled in a streaming fashion to minimize memory usage

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built using Rust and various awesome crates:
  - clap
  - rayon
  - csv
  - unicode-segmentation

## Version History

- 0.1.0 (2024-12-16)
  - Initial release
  - Basic duplicate detection functionality
  - Threshold filtering support

## Support

For bugs, feature requests, or questions, please open an issue on the GitHub repository.
