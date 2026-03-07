use crate::config::{Config, LogFormat};
use chrono::Local;
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

    let timestamp_format = config.logging.timestamp_format.clone();
    let timestamps_enabled = config.logging.timestamps;

    match config.logging.format {
        LogFormat::Json => {
            builder.format(|buf, record| {
                use std::io::Write;
                let json = serde_json::json!({
                    "timestamp": Local::now().to_rfc3339(),
                    "level": record.level().to_string(),
                    "target": record.target(),
                    "message": record.args().to_string()
                });
                writeln!(buf, "{}", json)
            });
        }
        LogFormat::Human => {
            if timestamps_enabled {
                builder.format(move |buf, record| {
                    use std::io::Write;
                    let timestamp = Local::now().format(&timestamp_format);
                    writeln!(
                        buf,
                        "[{}] {:<5} {}",
                        timestamp,
                        record.level(),
                        record.args()
                    )
                });
            } else {
                builder.format(|buf, record| {
                    use std::io::Write;
                    writeln!(buf, "{}", record.args())
                });
            }
        }
    }

    builder.init();
    Ok(())
}
