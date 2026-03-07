//! FRR Prefix List Generator
//! 
//! A Rust implementation of the FRR Prefix List Generator that:
//! - Extracts ASNs from FRR's BGP summary
//! - Fetches AS-SETs from PeeringDB API
//! - Generates prefix lists using bgpq4
//! - Applies them to FRR via vtysh
//! 
//! Architecture: Hexagonal/Clean Architecture with ports and adapters

pub mod adapters;
pub mod cli;
pub mod config;
pub mod diff;
pub mod error;
pub mod health;
pub mod logging;
pub mod ports;
pub mod service;
pub mod types;

use crate::adapters::{Bgpq4Adapter, PeeringDbAdapter, VtyshAdapter};
use crate::cli::Cli;
use crate::config::Config;
use crate::error::Result;
use crate::ports::{AsSetResolver, PrefixGenerator, RouterConfigurator};
use crate::service::PrefixListService;
use crate::types::Asn;
use std::sync::{Arc, Mutex};

/// Run the application with the given CLI arguments
pub fn run(cli: Cli) -> Result<()> {
    // Load configuration
    let config = load_config(&cli)?;
    
    // Initialize logging
    logging::init_logging(&config)?;
    log::info!("Starting FRR Prefix List Generator v{}", env!("CARGO_PKG_VERSION"));
    
    // Create adapters (infrastructure layer)
    let prefix_generator: Arc<dyn PrefixGenerator> = Arc::new(Bgpq4Adapter::new(config.bgpq4.clone()));
    let router_config: Arc<dyn RouterConfigurator> = Arc::new(VtyshAdapter::new());
    let as_set_resolver: Arc<Mutex<dyn AsSetResolver>> = Arc::new(Mutex::new(
        PeeringDbAdapter::new(config.peeringdb.clone(), config.general.api_timeout)
    ));
    
    // Create service (application layer)
    let service = PrefixListService::new(
        config,
        prefix_generator,
        router_config,
        as_set_resolver,
    );
    
    // Handle health check mode
    if cli.check {
        return run_health_check(&service);
    }
    
    // Discover ASNs to process
    let asns = if cli.asn.is_empty() {
        service.discover_asns()?
    } else {
        cli.asn.into_iter().filter_map(Asn::new).collect()
    };
    
    if asns.is_empty() {
        log::warn!("No ASNs found to process");
        return Ok(());
    }
    
    // Process ASNs
    service.process_asns(asns, cli.dry_run)?;
    
    log::info!("FRR Prefix List Generator completed successfully");
    Ok(())
}

fn load_config(cli: &Cli) -> Result<Config> {
    let mut config = if cli.config.exists() {
        Config::from_file(&cli.config)?
    } else {
        log::warn!("Config file not found at {:?}, using defaults", cli.config);
        Config::default()
    };
    
    // Apply CLI overrides
    if let Some(ref level) = cli.log_level {
        config.logging.level = level.clone();
    }
    
    if let Some(ref format) = cli.log_format {
        config.logging.format = match format.as_str() {
            "json" => config::LogFormat::Json,
            _ => config::LogFormat::Human,
        };
    }
    
    Ok(config)
}

fn run_health_check(service: &PrefixListService) -> Result<()> {
    log::info!("Running health checks...");
    
    match service.health_check() {
        Ok(true) => {
            log::info!("All health checks passed!");
            Ok(())
        }
        Ok(false) => {
            log::error!("Some health checks failed");
            std::process::exit(1);
        }
        Err(e) => {
            log::error!("Health check error: {}", e);
            std::process::exit(1);
        }
    }
}
