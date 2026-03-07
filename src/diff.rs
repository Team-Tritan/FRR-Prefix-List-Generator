//! Diff display utilities
//!
//! These functions handle the presentation of differences between prefix lists.
//! The actual diff calculation is now part of the PrefixLists type.

use crate::types::{Asn, PeerIPs, PrefixListDiff, PrefixLists};

/// Display a diff between current and new prefix lists
pub fn print_diff(asn: Asn, diff: &PrefixListDiff) {
    println!("\nDiff for {}", asn);

    if !diff.v4_added.is_empty() || !diff.v4_removed.is_empty() {
        println!("\nIPv4 Prefix Lists:");

        for line in &diff.v4_added {
            println!("  + {}", line);
        }

        for line in &diff.v4_removed {
            println!("  - {}", line);
        }

        println!(
            "  Summary: {} added, {} removed",
            diff.v4_added.len(),
            diff.v4_removed.len()
        );
    } else {
        println!("\nIPv4: No changes");
    }

    if !diff.v6_added.is_empty() || !diff.v6_removed.is_empty() {
        println!("\nIPv6 Prefix Lists:");

        for line in &diff.v6_added {
            println!("  + {}", line);
        }

        for line in &diff.v6_removed {
            println!("  - {}", line);
        }

        println!(
            "  Summary: {} added, {} removed",
            diff.v6_added.len(),
            diff.v6_removed.len()
        );
    } else {
        println!("\nIPv6: No changes");
    }

    if diff.has_changes() {
        println!("\nTotal: {} changes\n", diff.total_changes());
    }
}

/// Display a dry-run summary
pub fn print_dry_run_summary(asn: Asn, prefix_lists: &PrefixLists, peers: &PeerIPs) {
    println!("\nDry-run for {}", asn);

    println!(
        "  Prefix Lists: {} IPv4, {} IPv6",
        prefix_lists.v4_count(),
        prefix_lists.v6_count()
    );

    if !peers.v4_peers().is_empty() {
        println!(
            "  IPv4 Neighbors: {} peers with max-prefix {}",
            peers.v4_peers().len(),
            prefix_lists.v4_count()
        );
    }

    if !peers.v6_peers().is_empty() {
        println!(
            "  IPv6 Neighbors: {} peers with max-prefix {}",
            peers.v6_peers().len(),
            prefix_lists.v6_count()
        );
    }

    println!("\n[No changes applied - dry-run mode]");
}
