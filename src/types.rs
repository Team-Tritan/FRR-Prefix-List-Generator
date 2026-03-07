//! Domain types and value objects
//!
//! Following Domain-Driven Design principles, these types encapsulate
//! both data and behavior, ensuring invariants are maintained.

use std::collections::HashSet;
use std::fmt;

/// Autonomous System Number - validates range on construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Asn(u32);

impl Asn {
    /// Minimum valid ASN (public range)
    pub const MIN_PUBLIC: u32 = 1;
    /// Maximum valid ASN (32-bit range)
    pub const MAX: u32 = 4_294_967_295;
    /// Private ASN range start (16-bit)
    pub const PRIVATE_16BIT_START: u32 = 64_512;
    /// Private ASN range end (16-bit)
    pub const PRIVATE_16BIT_END: u32 = 65_534;
    /// Private ASN range start (32-bit)
    pub const PRIVATE_32BIT_START: u32 = 4_200_000_000;
    /// Private ASN range end (32-bit)
    pub const PRIVATE_32BIT_END: u32 = 4_294_967_294;

    /// Create a new ASN, validating it's in valid range
    pub fn new(asn: u32) -> Option<Self> {
        if asn == 0 { None } else { Some(Self(asn)) }
    }

    /// Create an ASN without validation (use with caution)
    pub const fn new_unchecked(asn: u32) -> Self {
        Self(asn)
    }

    /// Get the raw ASN number
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if this is a valid public ASN
    pub const fn is_valid_public(&self) -> bool {
        let is_private_16bit =
            self.0 >= Self::PRIVATE_16BIT_START && self.0 <= Self::PRIVATE_16BIT_END;
        let is_private_32bit =
            self.0 >= Self::PRIVATE_32BIT_START && self.0 <= Self::PRIVATE_32BIT_END;

        !is_private_16bit && !is_private_32bit
    }

    /// Check if this is a private ASN
    pub const fn is_private(&self) -> bool {
        !self.is_valid_public()
    }

    /// Generate prefix list name for this ASN
    pub fn prefix_list_name(&self, version: IpVersion) -> String {
        match version {
            IpVersion::V4 => format!("AS{}-In-v4", self.0),
            IpVersion::V6 => format!("AS{}-In-v6", self.0),
        }
    }
}

impl fmt::Display for Asn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AS{}", self.0)
    }
}

/// IP Protocol Version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    V4,
    V6,
}

/// Immutable collection of prefix list entries with deduplication
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrefixLists {
    v4: Vec<String>,
    v6: Vec<String>,
}

impl PrefixLists {
    /// Create empty prefix lists
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from raw vectors (takes ownership)
    pub fn from_entries(v4: Vec<String>, v6: Vec<String>) -> Self {
        let mut lists = Self { v4, v6 };
        lists.deduplicate();
        lists
    }

    /// Check if both IPv4 and IPv6 lists are empty
    pub fn is_empty(&self) -> bool {
        self.v4.is_empty() && self.v6.is_empty()
    }

    /// Total count of all prefixes
    pub fn len(&self) -> usize {
        self.v4.len() + self.v6.len()
    }

    /// Count of IPv4 prefixes
    pub fn v4_count(&self) -> usize {
        self.v4.len()
    }

    /// Count of IPv6 prefixes
    pub fn v6_count(&self) -> usize {
        self.v6.len()
    }

    /// Get immutable reference to IPv4 entries
    pub fn v4_entries(&self) -> &[String] {
        &self.v4
    }

    /// Get immutable reference to IPv6 entries
    pub fn v6_entries(&self) -> &[String] {
        &self.v6
    }

    /// Add entries from another PrefixLists (consumes other)
    pub fn merge(&mut self, other: Self) {
        self.v4.extend(other.v4);
        self.v6.extend(other.v6);
        self.deduplicate();
    }

    /// Add a single IPv4 entry
    pub fn add_v4(&mut self, entry: String) {
        if !self.v4.contains(&entry) {
            self.v4.push(entry);
        }
    }

    /// Add a single IPv6 entry
    pub fn add_v6(&mut self, entry: String) {
        if !self.v6.contains(&entry) {
            self.v6.push(entry);
        }
    }

    /// Sort and deduplicate all entries
    fn deduplicate(&mut self) {
        self.v4.sort_unstable();
        self.v4.dedup();
        self.v6.sort_unstable();
        self.v6.dedup();
    }

    /// Calculate difference between this and another PrefixLists
    pub fn diff(&self, other: &Self) -> PrefixListDiff {
        let v4_current: HashSet<_> = self.v4.iter().collect();
        let v4_new: HashSet<_> = other.v4.iter().collect();
        let v6_current: HashSet<_> = self.v6.iter().collect();
        let v6_new: HashSet<_> = other.v6.iter().collect();

        PrefixListDiff {
            v4_added: v4_new.difference(&v4_current).cloned().cloned().collect(),
            v4_removed: v4_current.difference(&v4_new).cloned().cloned().collect(),
            v6_added: v6_new.difference(&v6_current).cloned().cloned().collect(),
            v6_removed: v6_current.difference(&v6_new).cloned().cloned().collect(),
        }
    }
}

/// Represents changes between two PrefixLists
#[derive(Debug, Clone, Default)]
pub struct PrefixListDiff {
    pub v4_added: Vec<String>,
    pub v4_removed: Vec<String>,
    pub v6_added: Vec<String>,
    pub v6_removed: Vec<String>,
}

impl PrefixListDiff {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.v4_added.is_empty()
            || !self.v4_removed.is_empty()
            || !self.v6_added.is_empty()
            || !self.v6_removed.is_empty()
    }

    /// Total number of changes
    pub fn total_changes(&self) -> usize {
        self.v4_added.len() + self.v4_removed.len() + self.v6_added.len() + self.v6_removed.len()
    }
}

/// Peer IP addresses grouped by protocol version
#[derive(Debug, Clone, Default)]
pub struct PeerIPs {
    v4: Vec<String>,
    v6: Vec<String>,
}

impl PeerIPs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_ips(v4: Vec<String>, v6: Vec<String>) -> Self {
        Self { v4, v6 }
    }

    pub fn is_empty(&self) -> bool {
        self.v4.is_empty() && self.v6.is_empty()
    }

    pub fn v4_peers(&self) -> &[String] {
        &self.v4
    }

    pub fn v6_peers(&self) -> &[String] {
        &self.v6
    }

    pub fn add_v4(&mut self, ip: String) {
        if !self.v4.contains(&ip) {
            self.v4.push(ip);
        }
    }

    pub fn add_v6(&mut self, ip: String) {
        if !self.v6.contains(&ip) {
            self.v6.push(ip);
        }
    }
}

/// BGP neighbor information
#[derive(Debug, Clone)]
pub struct BgpNeighbor {
    pub ip: String,
    pub remote_as: Asn,
    pub state: String,
}

impl BgpNeighbor {
    pub fn new(ip: String, remote_as: Asn, state: String) -> Self {
        Self {
            ip,
            remote_as,
            state,
        }
    }

    /// Check if this neighbor is in an established state
    pub fn is_established(&self) -> bool {
        self.state.eq_ignore_ascii_case("established")
    }
}

/// AS-SET information
#[derive(Debug, Clone)]
pub struct AsSetInfo {
    pub name: String,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asn_validates_range() {
        assert!(Asn::new(0).is_none());
        assert!(Asn::new(1).is_some());
        assert!(Asn::new(100).is_some());
    }

    #[test]
    fn asn_detects_private() {
        let public = Asn::new(13335).unwrap();
        assert!(public.is_valid_public());
        assert!(!public.is_private());

        let private_16 = Asn::new(65000).unwrap();
        assert!(!private_16.is_valid_public());
        assert!(private_16.is_private());

        let private_32 = Asn::new(4_200_000_001).unwrap();
        assert!(!private_32.is_valid_public());
        assert!(private_32.is_private());
    }

    #[test]
    fn prefix_lists_deduplicates() {
        let mut lists = PrefixLists::new();
        lists.add_v4("entry1".to_string());
        lists.add_v4("entry1".to_string()); // duplicate
        lists.add_v4("entry2".to_string());

        assert_eq!(lists.v4_count(), 2);
    }

    #[test]
    fn prefix_lists_diff_calculates_correctly() {
        let mut current = PrefixLists::new();
        current.add_v4("keep".to_string());
        current.add_v4("remove".to_string());

        let mut new = PrefixLists::new();
        new.add_v4("keep".to_string());
        new.add_v4("add".to_string());

        let diff = current.diff(&new);

        assert_eq!(diff.v4_added, vec!["add".to_string()]);
        assert_eq!(diff.v4_removed, vec!["remove".to_string()]);
        assert!(diff.has_changes());
    }
}
