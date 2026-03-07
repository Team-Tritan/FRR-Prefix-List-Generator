//! Configuration validation module
//!
//! Provides comprehensive validation of configuration files
//! without requiring external service connectivity.

use crate::config::Config;
use crate::error::Result;
use std::fmt;

/// Validation result containing errors and warnings
#[derive(Debug)]
pub struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "✗ {}", error)?;
        }
        for warning in &self.warnings {
            writeln!(f, "⚠ {}", warning)?;
        }
        Ok(())
    }
}

/// Validates a configuration file at the given path
pub fn validate_config(path: &std::path::Path) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Check file exists and is readable
    if !path.exists() {
        result.add_error(format!("Config file not found: {}", path.display()));
        return Ok(result);
    }

    if !path.is_file() {
        result.add_error(format!("Config path is not a file: {}", path.display()));
        return Ok(result);
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            result.add_error(format!("Cannot read config file: {}", e));
            return Ok(result);
        }
    };

    // Try to parse as TOML
    let config: Config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            result.add_error(format!("Invalid TOML syntax: {}", e));
            return Ok(result);
        }
    };

    // Validate general config
    validate_general(&config, &mut result);

    // Validate PeeringDB config
    validate_peeringdb(&config, &mut result);

    // Validate logging config
    validate_logging(&config, &mut result);

    // Validate filter config
    validate_filter(&config, &mut result);

    // Validate bgpq4 config
    validate_bgpq4(&config, &mut result);

    Ok(result)
}

fn validate_general(config: &Config, result: &mut ValidationResult) {
    if config.general.concurrency == 0 {
        result.add_error("general.concurrency must be at least 1");
    } else if config.general.concurrency > 64 {
        result.add_warning(format!(
            "general.concurrency is very high ({}), may cause resource issues",
            config.general.concurrency
        ));
    }

    if config.general.bgpq4_timeout == 0 {
        result.add_error("general.bgpq4_timeout must be at least 1 second");
    } else if config.general.bgpq4_timeout > 300 {
        result.add_warning(format!(
            "general.bgpq4_timeout is very high ({}s)",
            config.general.bgpq4_timeout
        ));
    }

    if config.general.api_timeout == 0 {
        result.add_error("general.api_timeout must be at least 1 second");
    } else if config.general.api_timeout > 300 {
        result.add_warning(format!(
            "general.api_timeout is very high ({}s)",
            config.general.api_timeout
        ));
    }
}

fn validate_peeringdb(config: &Config, result: &mut ValidationResult) {
    // Validate base URL
    if config.peeringdb.base_url.is_empty() {
        result.add_error("peeringdb.base_url cannot be empty");
    } else if !config.peeringdb.base_url.starts_with("http://")
        && !config.peeringdb.base_url.starts_with("https://")
    {
        result.add_error("peeringdb.base_url must start with http:// or https://");
    } else if !config.peeringdb.base_url.starts_with("https://") {
        result.add_warning("peeringdb.base_url should use HTTPS for security");
    }

    // Validate rate limit
    if config.peeringdb.rate_limit_per_minute == 0 {
        result.add_error("peeringdb.rate_limit_per_minute must be at least 1");
    } else if config.peeringdb.rate_limit_per_minute > 1000 {
        result.add_warning(format!(
            "peeringdb.rate_limit_per_minute is very high ({}), may trigger rate limiting",
            config.peeringdb.rate_limit_per_minute
        ));
    }

    // Validate retries
    if config.peeringdb.max_retries == 0 {
        result.add_error("peeringdb.max_retries must be at least 1");
    } else if config.peeringdb.max_retries > 10 {
        result.add_warning(format!(
            "peeringdb.max_retries is very high ({}), consider lower value",
            config.peeringdb.max_retries
        ));
    }

    if config.peeringdb.retry_delay_secs == 0 {
        result.add_error("peeringdb.retry_delay_secs must be at least 1");
    } else if config.peeringdb.retry_delay_secs > 60 {
        result.add_warning(format!(
            "peeringdb.retry_delay_secs is very high ({}s)",
            config.peeringdb.retry_delay_secs
        ));
    }
}

fn validate_logging(config: &Config, result: &mut ValidationResult) {
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    let level_lower = config.logging.level.to_lowercase();

    if !valid_levels.contains(&level_lower.as_str()) {
        result.add_error(format!(
            "logging.level must be one of: trace, debug, info, warn, error (got: {})",
            config.logging.level
        ));
    } else if level_lower == "trace" || level_lower == "debug" {
        result.add_warning(format!(
            "logging.level is set to '{}' which may produce verbose output",
            config.logging.level
        ));
    }
}

fn validate_filter(config: &Config, result: &mut ValidationResult) {
    for (i, asn) in config.filter.ignore_asns.iter().enumerate() {
        if *asn == 0 {
            result.add_error(format!("filter.ignore_asns[{}] cannot be 0", i));
        } else if is_private_asn(*asn) {
            result.add_warning(format!(
                "filter.ignore_asns[{}] is a private ASN ({})",
                i, asn
            ));
        }
    }
}

fn validate_bgpq4(config: &Config, result: &mut ValidationResult) {
    if config.bgpq4.sources.is_empty() {
        result.add_error("bgpq4.sources cannot be empty");
    } else {
        let valid_sources = [
            "ARIN",
            "RIPE",
            "AFRINIC",
            "APNIC",
            "LACNIC",
            "RADB",
            "ALTDB",
            "RIPE-NONAUTH",
            "BELL",
            "JPIRR",
            "LEVEL3",
            "NESTEGG",
            "NTTCOM",
            "OPENFACE",
            "OSCARS",
            "RGNET",
            "ROGERS",
            "TC",
            "WCGDB",
        ];

        for (i, source) in config.bgpq4.sources.iter().enumerate() {
            if source.is_empty() {
                result.add_error(format!("bgpq4.sources[{}] cannot be empty", i));
                continue;
            }

            if !valid_sources.contains(&source.to_uppercase().as_str()) {
                result.add_warning(format!(
                    "bgpq4.sources[{}] is not a well-known IRR source: {}",
                    i, source
                ));
            }
        }

        // Warn if only one source
        if config.bgpq4.sources.len() == 1 {
            result.add_warning(
                "Only one bgpq4 source configured - consider using multiple for redundancy",
            );
        }
    }
}

fn is_private_asn(asn: u32) -> bool {
    // RFC 6996 private ASN ranges
    (64512..=65534).contains(&asn) || (4200000000..=4294967294).contains(&asn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_result_tracks_errors() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());

        result.add_error("test error");
        assert!(!result.is_valid());
        assert_eq!(result.errors().len(), 1);
    }

    #[test]
    fn validation_result_tracks_warnings() {
        let mut result = ValidationResult::new();
        assert!(!result.has_warnings());

        result.add_warning("test warning");
        assert!(result.has_warnings());
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn is_private_asn_detects_private_ranges() {
        assert!(is_private_asn(64512)); // Start of 16-bit private range
        assert!(is_private_asn(65534)); // End of 16-bit private range
        assert!(is_private_asn(4200000000)); // Start of 32-bit private range
        assert!(is_private_asn(4294967294)); // End of 32-bit private range
        assert!(!is_private_asn(393577)); // Public ASN
        assert!(!is_private_asn(1)); // Start of public range
        assert!(!is_private_asn(64511)); // Just before private range
        assert!(!is_private_asn(65535)); // Reserved, not private
    }
}
