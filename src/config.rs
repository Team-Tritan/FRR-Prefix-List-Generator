use crate::error::{PrefixGenError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub peeringdb: PeeringDbConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub filter: FilterConfig,
    #[serde(default)]
    pub bgpq4: Bgpq4Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_bgpq4_timeout")]
    pub bgpq4_timeout: u64,
    #[serde(default = "default_api_timeout")]
    pub api_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeeringDbConfig {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_format")]
    pub format: LogFormat,
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterConfig {
    #[serde(default)]
    pub ignore_asns: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bgpq4Config {
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Human,
    Json,
}

// Default functions
fn default_concurrency() -> usize {
    4
}
fn default_bgpq4_timeout() -> u64 {
    10
}
fn default_api_timeout() -> u64 {
    30
}
fn default_base_url() -> String {
    "https://www.peeringdb.com/api".to_string()
}
fn default_rate_limit() -> u32 {
    60
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay() -> u64 {
    5
}
fn default_log_format() -> LogFormat {
    LogFormat::Human
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_sources() -> Vec<String> {
    vec![
        "ARIN".to_string(),
        "RIPE".to_string(),
        "AFRINIC".to_string(),
        "APNIC".to_string(),
        "LACNIC".to_string(),
        "RADB".to_string(),
        "ALTDB".to_string(),
    ]
}

// Default implementations
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            bgpq4_timeout: default_bgpq4_timeout(),
            api_timeout: default_api_timeout(),
        }
    }
}

impl Default for PeeringDbConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            rate_limit_per_minute: default_rate_limit(),
            max_retries: default_max_retries(),
            retry_delay_secs: default_retry_delay(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: default_log_format(),
            level: default_log_level(),
        }
    }
}

impl Default for Bgpq4Config {
    fn default() -> Self {
        Self {
            sources: default_sources(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            peeringdb: PeeringDbConfig::default(),
            logging: LoggingConfig::default(),
            filter: FilterConfig::default(),
            bgpq4: Bgpq4Config::default(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| PrefixGenError::ConfigError(format!("Failed to parse config: {}", e)))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.general.concurrency == 0 {
            return Err(PrefixGenError::ConfigError(
                "Concurrency must be at least 1".to_string(),
            ));
        }

        if self.peeringdb.rate_limit_per_minute == 0 {
            return Err(PrefixGenError::ConfigError(
                "Rate limit must be at least 1 req/min".to_string(),
            ));
        }

        Ok(())
    }
}
