//! External service interfaces (Ports in Hexagonal Architecture)
//!
//! These traits define the boundaries between our domain logic and external systems.
//! They allow for easy testing via mocks and make dependencies explicit.

use crate::error::Result;
use crate::types::{Asn, PrefixLists};

/// Interface for BGP prefix list generation via bgpq4
pub trait PrefixGenerator: Send + Sync {
    /// Check if bgpq4 is available and working
    fn health_check(&self) -> Result<()>;

    /// Generate prefix lists for an ASN using its AS-SETs
    fn generate_prefix_lists(&self, asn: Asn, as_sets: &[String]) -> Result<PrefixLists>;
}

/// Interface for FRR/VTYSH operations
pub trait RouterConfigurator: Send + Sync {
    /// Check if vtysh is available
    fn health_check(&self) -> Result<()>;

    /// Get current BGP neighbors and their ASNs
    fn get_bgp_neighbors(&self) -> Result<Vec<(String, Asn)>>;

    /// Get peer IPs for a specific ASN
    fn get_peer_ips(&self, asn: Asn) -> Result<(Vec<String>, Vec<String>)>;

    /// Apply prefix lists to router configuration
    fn apply_prefix_lists(&self, asn: Asn, prefix_lists: &PrefixLists) -> Result<()>;

    /// Get existing prefix lists for comparison
    fn get_current_prefix_lists(&self, asn: Asn) -> Result<PrefixLists>;

    /// Set maximum prefix limits for neighbors
    fn set_max_prefix_limits(
        &self,
        asn: Asn,
        v4_peers: &[String],
        v6_peers: &[String],
        v4_count: usize,
        v6_count: usize,
    ) -> Result<()>;
}

/// Interface for AS-SET lookup services
pub trait AsSetResolver: Send + Sync {
    /// Check connectivity to the resolver
    fn health_check(&mut self) -> Result<()>;

    /// Fetch AS-SET names for a given ASN
    fn fetch_as_sets(&mut self, asn: Asn) -> Result<Vec<String>>;
}
