use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "frr-prefix-gen")]
#[command(about = "FRR Prefix List Generator - Fetches AS-SETs and generates BGP prefix lists")]
#[command(version)]
pub struct Cli {
    /// Path to configuration file
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "/etc/frr-prefix-gen/config.toml"
    )]
    pub config: PathBuf,

    /// Run in dry-run mode (show what would be done without making changes)
    #[arg(long)]
    pub dry_run: bool,

    /// Run health check and exit
    #[arg(long)]
    pub check: bool,

    /// Override log level (trace, debug, info, warn, error)
    #[arg(short, long)]
    pub log_level: Option<String>,

    /// Override log format (human, json)
    #[arg(long)]
    pub log_format: Option<String>,

    /// Only process specific ASN (can be specified multiple times)
    #[arg(short, long)]
    pub asn: Vec<u32>,

    /// Validate configuration file and exit
    #[arg(long)]
    pub validate: bool,

    /// Treat warnings as errors (only applies with --validate)
    #[arg(long)]
    pub strict: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
