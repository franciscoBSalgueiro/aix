use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// Path to the input file, PGN or zstd-compressed PGN (.zst)
    #[arg(short, long)]
    pub input: String,

    /// Path to the output file(s) (without extension)
    #[arg(short, long)]
    pub output: String,

    /// Optional DuckDB memory limit in GB
    #[arg(long)]
    pub duckdb_memory_limit_gb: Option<u16>,

    /// Set this flag to continue processing even if an invalid move is encountered in a game, rather than exiting with an error. The game with the invalid move will end right before the invalid move.
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub continue_on_invalid_move: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(super) enum CompressionLevel {
    Low,
    Medium,
    High,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
pub(super) enum ParquetCompression {
    Uncompressed,
    Snappy,
    Gzip,
    Zstd,
    Brotli,
    Lz4Raw,
}

impl ParquetCompression {
    pub fn as_str(&self) -> &str {
        match self {
            ParquetCompression::Uncompressed => "uncompressed",
            ParquetCompression::Snappy => "snappy",
            ParquetCompression::Gzip => "gzip",
            ParquetCompression::Zstd => "zstd",
            ParquetCompression::Brotli => "brotli",
            ParquetCompression::Lz4Raw => "lz4_raw",
        }
    }
}
