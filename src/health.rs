//! Health check utilities
//!
//! These are kept for backward compatibility but the main health checking
//! logic has been moved into the PrefixListService.

use crate::error::Result;
use crate::ports::{AsSetResolver, PrefixGenerator, RouterConfigurator};
use std::sync::{Arc, Mutex};

/// Run health check with explicit dependencies
///
/// Note: Prefer using PrefixListService::health_check() instead
pub fn run_health_check(
    prefix_gen: Arc<dyn PrefixGenerator>,
    router: Arc<dyn RouterConfigurator>,
    resolver: Arc<Mutex<dyn AsSetResolver>>,
) -> Result<bool> {
    let mut all_passed = true;

    if let Err(e) = prefix_gen.health_check() {
        log::error!("Prefix generator check failed: {}", e);
        all_passed = false;
    }

    if let Err(e) = router.health_check() {
        log::error!("Router config check failed: {}", e);
        all_passed = false;
    }

    if let Ok(mut resolver) = resolver.lock()
        && let Err(e) = resolver.health_check()
    {
        log::error!("AS-SET resolver check failed: {}", e);
        all_passed = false;
    }

    Ok(all_passed)
}
