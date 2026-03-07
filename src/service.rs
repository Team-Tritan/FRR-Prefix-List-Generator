//! Application service layer implementing the core business logic
//! 
//! This module provides high-level operations that coordinate between
//! external services (adapters) while remaining independent of their implementations.

use crate::config::Config;
use crate::error::Result;
use crate::ports::{AsSetResolver, PrefixGenerator, RouterConfigurator};
use crate::types::{Asn, PeerIPs, PrefixLists};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

/// Orchestrates the prefix list generation workflow
pub struct PrefixListService {
    config: Arc<Config>,
    prefix_generator: Arc<dyn PrefixGenerator>,
    router_config: Arc<dyn RouterConfigurator>,
    as_set_resolver: Arc<Mutex<dyn AsSetResolver>>,
}

impl PrefixListService {
    pub fn new(
        config: Config,
        prefix_generator: Arc<dyn PrefixGenerator>,
        router_config: Arc<dyn RouterConfigurator>,
        as_set_resolver: Arc<Mutex<dyn AsSetResolver>>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            prefix_generator,
            router_config,
            as_set_resolver,
        }
    }
    
    /// Run health checks on all dependencies
    pub fn health_check(&self) -> Result<bool> {
        log::info!("Running health checks");
        
        let mut all_passed = true;
        
        if let Err(e) = self.router_config.health_check() {
            log::error!("Router configurator health check failed: {}", e);
            all_passed = false;
        }
        
        if let Err(e) = self.prefix_generator.health_check() {
            log::error!("Prefix generator health check failed: {}", e);
            all_passed = false;
        }
        
        if let Ok(mut resolver) = self.as_set_resolver.lock() {
            if let Err(e) = resolver.health_check() {
                log::error!("AS-SET resolver health check failed: {}", e);
                all_passed = false;
            }
        }
        
        if all_passed {
            log::info!("All health checks passed");
        }
        
        Ok(all_passed)
    }
    
    /// Discover ASNs to process from BGP neighbors
    pub fn discover_asns(&self) -> Result<Vec<Asn>> {
        log::info!("Discovering ASNs from BGP neighbors");
        
        let neighbors = self.router_config.get_bgp_neighbors()?;
        let ignore_list: HashSet<u32> = self.config.filter.ignore_asns.iter().cloned().collect();
        
        let mut asns: Vec<Asn> = neighbors
            .into_iter()
            .filter_map(|(_, asn)| {
                if !ignore_list.contains(&asn.as_u32()) {
                    Some(asn)
                } else {
                    None
                }
            })
            .collect();
        
        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        asns.retain(|asn| seen.insert(asn.as_u32()));
        
        // Sort for deterministic ordering
        asns.sort_by_key(|a| a.as_u32());
        
        log::info!("Discovered {} unique ASNs to process", asns.len());
        
        Ok(asns)
    }
    
    /// Process a single ASN end-to-end
    pub fn process_asn(&self, asn: Asn, dry_run: bool) -> Result<()> {
        log::info!("Processing {}", asn);
        
        // Step 1: Fetch AS-SETs
        let as_sets = self.fetch_as_sets(asn)?;
        log::info!("{} has {} AS-SET(s): {:?}", asn, as_sets.len(), as_sets);
        
        // Step 2: Generate prefix lists
        let new_prefix_lists = self.generate_prefix_lists(asn, &as_sets)?;
        
        if new_prefix_lists.is_empty() {
            log::warn!("No prefix lists generated for {}", asn);
            return Ok(());
        }
        
        // Step 3: Get peer IPs
        let (v4_peers, v6_peers) = self.router_config.get_peer_ips(asn)?;
        let peers = PeerIPs::from_ips(v4_peers, v6_peers);
        
        // Step 4: Show diff in dry-run mode
        if dry_run {
            self.show_diff(asn, &new_prefix_lists)?;
            self.show_summary(asn, &new_prefix_lists, &peers);
        }
        
        // Step 5: Apply prefix lists
        self.router_config.apply_prefix_lists(asn, &new_prefix_lists)?;
        
        // Step 6: Set max-prefix limits
        if let Err(e) = self.router_config.set_max_prefix_limits(
            peers.v4_peers(),
            peers.v6_peers(),
            new_prefix_lists.v4_count(),
            new_prefix_lists.v6_count(),
        ) {
            log::error!("Failed to set max-prefix limits for {}: {}", asn, e);
        }
        
        log::info!("Successfully processed {}", asn);
        Ok(())
    }
    
    /// Process multiple ASNs in parallel
    pub fn process_asns(&self, asns: Vec<Asn>, dry_run: bool) -> Result<()> {
        log::info!("Processing {} ASNs with concurrency {}", asns.len(), self.config.general.concurrency);
        
        let asns = Arc::new(Mutex::new(asns));
        let mut handles = vec![];
        
        for _ in 0..self.config.general.concurrency {
            let asns_clone = Arc::clone(&asns);
            let service = self.clone();
            
            let handle = thread::spawn(move || {
                loop {
                    let asn = {
                        let mut lock = asns_clone.lock().unwrap();
                        lock.pop()
                    };
                    
                    match asn {
                        Some(asn) => {
                            if let Err(e) = service.process_asn(asn, dry_run) {
                                log::error!("Failed to process {}: {}", asn, e);
                            }
                        }
                        None => break,
                    }
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().expect("Worker thread panicked");
        }
        
        log::info!("All ASNs processed");
        Ok(())
    }
    
    fn fetch_as_sets(&self, asn: Asn) -> Result<Vec<String>> {
        let mut resolver = self.as_set_resolver.lock().unwrap();
        resolver.fetch_as_sets(asn)
    }
    
    fn generate_prefix_lists(&self, asn: Asn, as_sets: &[String]) -> Result<PrefixLists> {
        self.prefix_generator.generate_prefix_lists(asn, as_sets)
    }
    
    fn show_diff(&self, asn: Asn, new_lists: &PrefixLists) -> Result<()> {
        let current = self.router_config.get_current_prefix_lists(asn)?;
        let diff = current.diff(new_lists);
        
        if diff.has_changes() {
            log::info!("Changes for {}:", asn);
            
            for entry in &diff.v4_added {
                log::info!("  + {}", entry);
            }
            for entry in &diff.v4_removed {
                log::info!("  - {}", entry);
            }
            for entry in &diff.v6_added {
                log::info!("  + {}", entry);
            }
            for entry in &diff.v6_removed {
                log::info!("  - {}", entry);
            }
            
            log::info!("  Total: {} changes", diff.total_changes());
        } else {
            log::info!("No changes for {}", asn);
        }
        
        Ok(())
    }
    
    fn show_summary(&self, asn: Asn, prefix_lists: &PrefixLists, peers: &PeerIPs) {
        log::info!("Dry-run summary for {}:", asn);
        log::info!("  Prefix Lists: {} IPv4, {} IPv6", prefix_lists.v4_count(), prefix_lists.v6_count());

        if !peers.v4_peers().is_empty() {
            log::info!("  IPv4 Neighbors: {} peers with max-prefix {}", peers.v4_peers().len(), prefix_lists.v4_count());
        }

        if !peers.v6_peers().is_empty() {
            log::info!("  IPv6 Neighbors: {} peers with max-prefix {}", peers.v6_peers().len(), prefix_lists.v6_count());
        }
    }
}

// Manual Clone implementation for the service
impl Clone for PrefixListService {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            prefix_generator: Arc::clone(&self.prefix_generator),
            router_config: Arc::clone(&self.router_config),
            as_set_resolver: Arc::clone(&self.as_set_resolver),
        }
    }
}
