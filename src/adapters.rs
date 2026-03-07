//! Implementation of external service adapters
//!
//! These adapters implement the port interfaces and wrap the actual
//! external tools (bgpq4, vtysh, PeeringDB API).

use crate::config::{Bgpq4Config, PeeringDbConfig};
use crate::error::{PrefixGenError, Result};
use crate::ports::{AsSetResolver, PrefixGenerator, RouterConfigurator};
use crate::types::{Asn, IpVersion, PrefixLists};
use regex::Regex;
use serde::Deserialize;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Bgpq4 command wrapper implementing PrefixGenerator
pub struct Bgpq4Adapter {
    config: Bgpq4Config,
}

impl Bgpq4Adapter {
    pub fn new(config: Bgpq4Config) -> Self {
        Self { config }
    }
}

impl PrefixGenerator for Bgpq4Adapter {
    fn health_check(&self) -> Result<()> {
        Command::new("bgpq4")
            .arg("--version")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    PrefixGenError::Bgpq4NotFound
                } else {
                    PrefixGenError::Bgpq4Error {
                        as_set: "check".to_string(),
                        reason: format!("Failed to run bgpq4: {}", e),
                    }
                }
            })
            .map(|_| ())
    }

    fn generate_prefix_lists(&self, asn: Asn, as_sets: &[String]) -> Result<PrefixLists> {
        let mut results = PrefixLists::new();
        let sources_str = self.config.sources.join(",");

        for as_set in as_sets {
            log::info!("Generating prefix lists for {} via {}", asn, as_set);

            let v4_name = asn.prefix_list_name(IpVersion::V4);
            let v6_name = asn.prefix_list_name(IpVersion::V6);

            // Run IPv4 and IPv6 in parallel
            let sources_str_v4 = sources_str.clone();
            let sources_str_v6 = sources_str.clone();
            let as_set_v4 = as_set.clone();
            let as_set_v6 = as_set.clone();

            let v4_handle = thread::spawn(move || {
                run_bgpq4(&as_set_v4, &v4_name, &sources_str_v4, IpVersion::V4)
            });

            let v6_handle = thread::spawn(move || {
                run_bgpq4(&as_set_v6, &v6_name, &sources_str_v6, IpVersion::V6)
            });

            let v4_result = v4_handle
                .join()
                .expect("IPv4 bgpq4 thread panicked unexpectedly");
            match v4_result {
                Ok(lines) => {
                    log::debug!(
                        "Parsed {} IPv4 prefix-list lines for {}",
                        lines.len(),
                        as_set
                    );
                    for line in lines {
                        results.add_v4(line);
                    }
                }
                Err(e) => {
                    log::error!("Failed to generate IPv4 prefixes for {}: {}", as_set, e);
                }
            }

            let v6_result = v6_handle
                .join()
                .expect("IPv6 bgpq4 thread panicked unexpectedly");
            match v6_result {
                Ok(lines) => {
                    log::debug!(
                        "Parsed {} IPv6 prefix-list lines for {}",
                        lines.len(),
                        as_set
                    );
                    for line in lines {
                        results.add_v6(line);
                    }
                }
                Err(e) => {
                    log::error!("Failed to generate IPv6 prefixes for {}: {}", as_set, e);
                }
            }
        }

        log::info!(
            "Generated {} total prefixes for {} ({} v4, {} v6)",
            results.len(),
            asn,
            results.v4_count(),
            results.v6_count()
        );

        Ok(results)
    }
}

fn run_bgpq4(
    as_set: &str,
    list_name: &str,
    sources: &str,
    version: IpVersion,
) -> Result<Vec<String>> {
    let mut cmd = Command::new("bgpq4");

    cmd.arg(as_set)
        .arg("-l")
        .arg(list_name)
        .arg("-S")
        .arg(sources);

    if version == IpVersion::V6 {
        cmd.arg("-6");
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = cmd.output().map_err(|e| PrefixGenError::Bgpq4Error {
        as_set: as_set.to_string(),
        reason: format!("Failed to execute: {}", e),
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty() && !s.starts_with("no "))
            .collect())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(PrefixGenError::Bgpq4Error {
            as_set: as_set.to_string(),
            reason: format!("Exit code {:?}: {}", output.status.code(), stderr),
        })
    }
}

/// Vtysh command wrapper implementing RouterConfigurator
pub struct VtyshAdapter;

impl Default for VtyshAdapter {
    fn default() -> Self {
        Self
    }
}

impl VtyshAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl RouterConfigurator for VtyshAdapter {
    fn health_check(&self) -> Result<()> {
        let output = Command::new("vtysh")
            .args(["-c", "show version"])
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    PrefixGenError::VtyshNotFound
                } else {
                    PrefixGenError::VtyshError(format!("Failed to execute vtysh: {}", e))
                }
            })?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            log::info!(
                "vtysh is available: {}",
                version.lines().next().unwrap_or("unknown")
            );
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PrefixGenError::VtyshError(format!(
                "vtysh error: {}",
                stderr
            )))
        }
    }

    fn get_bgp_neighbors(&self) -> Result<Vec<(String, Asn)>> {
        let output = self.execute_vtysh_command("show bgp summary")?;
        parse_bgp_summary(&output)
    }

    fn get_peer_ips(&self, asn: Asn) -> Result<(Vec<String>, Vec<String>)> {
        let output = self.execute_vtysh_command("show bgp summary")?;
        parse_peer_ips(&output, asn)
    }

    fn apply_prefix_lists(&self, asn: Asn, prefix_lists: &PrefixLists) -> Result<()> {
        if prefix_lists.is_empty() {
            log::warn!("No prefix-list commands to apply for {}", asn);
            return Ok(());
        }

        let commands = build_prefix_list_commands(prefix_lists);
        log::info!(
            "Applying {} prefix-list commands for {}",
            commands.len(),
            asn
        );

        self.execute_vtysh_commands(&commands)?;
        log::info!("Successfully applied prefix lists for {}", asn);

        Ok(())
    }

    fn get_current_prefix_lists(&self, asn: Asn) -> Result<PrefixLists> {
        let mut lists = PrefixLists::new();

        let v4_name = asn.prefix_list_name(IpVersion::V4);
        match self.get_prefix_list(&v4_name) {
            Ok(lines) => {
                for line in lines {
                    lists.add_v4(line);
                }
            }
            Err(e) => {
                log::debug!("No existing IPv4 prefix list for {}: {}", asn, e);
            }
        }

        let v6_name = asn.prefix_list_name(IpVersion::V6);
        match self.get_prefix_list(&v6_name) {
            Ok(lines) => {
                for line in lines {
                    lists.add_v6(line);
                }
            }
            Err(e) => {
                log::debug!("No existing IPv6 prefix list for {}: {}", asn, e);
            }
        }

        Ok(lists)
    }

    fn set_max_prefix_limits(
        &self,
        v4_peers: &[String],
        v6_peers: &[String],
        v4_count: usize,
        v6_count: usize,
    ) -> Result<()> {
        for peer in v4_peers {
            if v4_count > 0 {
                let commands = vec![
                    "configure terminal".to_string(),
                    "router bgp".to_string(),
                    "address-family ipv4 unicast".to_string(),
                    format!("neighbor {} maximum-prefix {}", peer, v4_count),
                    "end".to_string(),
                ];

                log::info!(
                    "Setting IPv4 maximum-prefix for neighbor {}: {}",
                    peer,
                    v4_count
                );
                self.execute_vtysh_commands(&commands)?;
            }
        }

        for peer in v6_peers {
            if v6_count > 0 {
                let commands = vec![
                    "configure terminal".to_string(),
                    "router bgp".to_string(),
                    "address-family ipv6 unicast".to_string(),
                    format!("neighbor {} maximum-prefix {}", peer, v6_count),
                    "end".to_string(),
                ];

                log::info!(
                    "Setting IPv6 maximum-prefix for neighbor {}: {}",
                    peer,
                    v6_count
                );
                self.execute_vtysh_commands(&commands)?;
            }
        }

        Ok(())
    }
}

impl VtyshAdapter {
    fn execute_vtysh_command(&self, command: &str) -> Result<String> {
        let output = Command::new("vtysh")
            .args(["-c", command])
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    PrefixGenError::VtyshNotFound
                } else {
                    PrefixGenError::VtyshError(format!("Failed to execute vtysh: {}", e))
                }
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PrefixGenError::VtyshError(stderr.to_string()))
        }
    }

    fn execute_vtysh_commands(&self, commands: &[String]) -> Result<()> {
        let mut args: Vec<String> = Vec::new();

        for cmd in commands {
            args.push("-c".to_string());
            args.push(cmd.clone());
        }

        log::debug!("Executing vtysh with {} commands", commands.len());

        let output = Command::new("vtysh")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PrefixGenError::VtyshError(format!("Failed to execute vtysh: {}", e)))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("vtysh command failed: {}", stderr);
            Err(PrefixGenError::VtyshError(stderr.to_string()))
        }
    }

    fn get_prefix_list(&self, name: &str) -> Result<Vec<String>> {
        let output = self.execute_vtysh_command(&format!("show ip prefix-list {}", name))?;
        Ok(output
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
}

fn parse_bgp_summary(output: &str) -> Result<Vec<(String, Asn)>> {
    let mut neighbors = Vec::new();

    for line in output.lines().skip(6) {
        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.len() >= 3
            && let Some(asn) = Asn::new(columns[2].parse::<u32>().unwrap_or(0))
        {
            neighbors.push((columns[0].to_string(), asn));
        }
    }

    Ok(neighbors)
}

fn parse_peer_ips(output: &str, target_asn: Asn) -> Result<(Vec<String>, Vec<String>)> {
    let mut v4_peers = Vec::new();
    let mut v6_peers = Vec::new();

    let re = Regex::new(r"^(\S+)\s+.*?(\d+)\s").unwrap();

    for line in output.lines().skip(6) {
        if let Some(caps) = re.captures(line) {
            let ip = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let remote_as = caps
                .get(2)
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .unwrap_or(0);

            if remote_as == target_asn.as_u32() {
                if ip.contains(':') {
                    if !v6_peers.contains(&ip.to_string()) {
                        v6_peers.push(ip.to_string());
                    }
                } else {
                    if !v4_peers.contains(&ip.to_string()) {
                        v4_peers.push(ip.to_string());
                    }
                }
            }
        }
    }

    log::debug!(
        "Found {} IPv4 and {} IPv6 peers for {}",
        v4_peers.len(),
        v6_peers.len(),
        target_asn
    );

    Ok((v4_peers, v6_peers))
}

fn build_prefix_list_commands(prefix_lists: &PrefixLists) -> Vec<String> {
    let mut commands = vec!["configure terminal".to_string()];

    for entry in prefix_lists.v4_entries() {
        commands.push(entry.clone());
    }

    for entry in prefix_lists.v6_entries() {
        commands.push(entry.clone());
    }

    commands.push("end".to_string());
    commands
}

/// PeeringDB API client implementing AsSetResolver
pub struct PeeringDbAdapter {
    config: PeeringDbConfig,
    timeout_secs: u64,
    last_request: Option<std::time::Instant>,
    min_interval: Duration,
}

#[derive(Debug, Deserialize)]
struct AsSetResponse {
    data: Vec<AsSetData>,
}

#[derive(Debug, Deserialize)]
struct AsSetData {
    #[serde(rename = "as_set")]
    as_set: Option<String>,
    #[serde(rename = "as_set_ixp")]
    as_set_ixp: Option<String>,
    #[serde(rename = "as_set_route_server")]
    as_set_route_server: Option<String>,
}

impl PeeringDbAdapter {
    pub fn new(config: PeeringDbConfig, timeout_secs: u64) -> Self {
        let min_interval = if config.rate_limit_per_minute > 0 {
            Duration::from_secs_f32(60.0 / config.rate_limit_per_minute as f32)
        } else {
            Duration::from_millis(0)
        };

        Self {
            config,
            timeout_secs,
            last_request: None,
            min_interval,
        }
    }

    fn apply_rate_limit(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < self.min_interval {
                let wait = self.min_interval - elapsed;
                thread::sleep(wait);
            }
        }
        self.last_request = Some(std::time::Instant::now());
    }
}

impl AsSetResolver for PeeringDbAdapter {
    fn health_check(&mut self) -> Result<()> {
        log::debug!("Running health check");
        let test_asn = Asn::new_unchecked(13335);
        match self.fetch_as_sets(test_asn) {
            Ok(_) => {
                log::info!("Health check passed");
                Ok(())
            }
            Err(e) => {
                log::warn!("Health check failed: {}", e);
                Err(e)
            }
        }
    }

    fn fetch_as_sets(&mut self, asn: Asn) -> Result<Vec<String>> {
        log::info!("Fetching AS-SETs for {}", asn);

        self.apply_rate_limit();

        let url = format!("{}/as_set/{}", self.config.base_url, asn.as_u32());
        let mut attempt = 0;
        let max_retries = self.config.max_retries;

        loop {
            attempt += 1;

            match minreq::get(&url)
                .with_header("User-Agent", "frr-prefix-gen/0.1.0")
                .with_timeout(self.timeout_secs)
                .send()
            {
                Ok(response) => {
                    let status = response.status_code;

                    if (200..300).contains(&status) {
                        return parse_as_set_response(response.as_str().unwrap_or("{}"), asn);
                    } else if status == 429 {
                        if attempt > max_retries {
                            log::warn!("Rate limit exceeded for {}, max retries reached", asn);
                            return Ok(vec![format!("AS{}", asn.as_u32())]);
                        }
                        let retry_after = 60u64;
                        log::warn!(
                            "Rate limited for {}, waiting {}s (attempt {}/{})",
                            asn,
                            retry_after,
                            attempt,
                            max_retries
                        );
                        thread::sleep(Duration::from_secs(retry_after));
                        continue;
                    } else {
                        return Err(PrefixGenError::PeeringDbError {
                            status: status as u16,
                            message: format!("HTTP error for {}", asn),
                        });
                    }
                }
                Err(e) => {
                    if attempt > max_retries {
                        log::warn!(
                            "Network error for {} after {} attempts, falling back to direct AS",
                            asn,
                            max_retries
                        );
                        return Ok(vec![format!("AS{}", asn.as_u32())]);
                    }
                    let retry_delay = 2u64.pow(attempt);
                    log::warn!(
                        "Network error for {}, retrying in {}s (attempt {}/{}): {}",
                        asn,
                        retry_delay,
                        attempt,
                        max_retries,
                        e
                    );
                    thread::sleep(Duration::from_secs(retry_delay));
                    continue;
                }
            }
        }
    }
}

fn parse_as_set_response(json: &str, asn: Asn) -> Result<Vec<String>> {
    match serde_json::from_str::<AsSetResponse>(json) {
        Ok(data) => {
            let as_sets = extract_as_set_names(&data.data);
            if !as_sets.is_empty() {
                log::info!(
                    "Found {} AS-SET(s) for {}: {:?}",
                    as_sets.len(),
                    asn,
                    as_sets
                );
                Ok(as_sets)
            } else {
                log::warn!("No AS-SETs found for {}, falling back to {}", asn, asn);
                Ok(vec![format!("AS{}", asn.as_u32())])
            }
        }
        Err(e) => {
            log::warn!("Failed to parse response for {}: {}", asn, e);
            Ok(vec![format!("AS{}", asn.as_u32())])
        }
    }
}

fn extract_as_set_names(data: &[AsSetData]) -> Vec<String> {
    let mut as_sets = Vec::new();

    for item in data {
        for name in [&item.as_set, &item.as_set_ixp, &item.as_set_route_server]
            .into_iter()
            .flatten()
        {
            if !name.is_empty() && !as_sets.contains(name) {
                as_sets.push(name.clone());
            }
        }
    }

    as_sets
}
