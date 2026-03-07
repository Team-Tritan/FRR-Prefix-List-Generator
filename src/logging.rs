use crate::config::{Config, LogFormat};
use env_logger::Builder;
use log::LevelFilter;

pub fn init_logging(config: &Config) -> crate::error::Result<()> {
    let level = match config.logging.level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let mut builder = Builder::new();
    builder.filter_level(level);

    match config.logging.format {
        LogFormat::Json => {
            builder.format(|buf, record| {
                use std::io::Write;
                let json = serde_json::json!({
                    "level": record.level().to_string(),
                    "target": record.target(),
                    "message": record.args().to_string()
                });
                writeln!(buf, "{}", json)
            });
        }
        LogFormat::Human => {
            builder.format(|buf, record| {
                use std::io::Write;
                writeln!(buf, "{} {}", record.target(), record.args())
            });
        }
    }

    builder.init();
    Ok(())
}
