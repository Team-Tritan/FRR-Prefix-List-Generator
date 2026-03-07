# FRR Prefix List Generator

Made with love by [AS393577](https://tritan.gg) <3

## What it does

This tool automatically generates BGP prefix lists for your FRR router. It reads your BGP neighbors, looks up their AS-SETs on PeeringDB, generates the prefix filters using bgpq4, and applies them directly to your running config.

The flow is pretty simple:

1. Read `show bgp summary` from FRR to find all your BGP neighbors
2. Query PeeringDB for each neighbor's AS-SET (or use their ASN directly if no AS-SET exists)
3. Generate prefix lists with bgpq4 - IPv4 lists get named `AS00000-In-v4`, IPv6 gets `AS00000-In-v6`
4. Apply the lists and set max-prefix limits on the neighbor sessions

You can ignore certain ASNs (IXPs, transits, whatever) via the config file.

## Why Rust?

The original version was TypeScript/Bun. It worked fine, but you needed Bun installed or had to bundle the runtime. This Rust version compiles to a single 4MB binary with zero dependencies. Just copy it to your router and run it.

## Installation

Download the binary from releases (or compile yourself with `cargo build --release`).

You'll need these tools on your system:
- `bgpq4` - for generating prefix lists from AS-SETs
- `vtysh` - FRR's CLI tool (you probably already have this)

## Configuration

Copy `config.example.toml` to `/etc/frr-prefix-gen/config.toml` and edit it:

```toml
[general]
concurrency = 4  # How many ASNs to process at once

[logging]
level = "info"
format = "human"
timestamps = false              # Enable timestamps in logs
timestamp_format = "%Y-%m-%d %H:%M:%S"  # strftime format

[filter]
ignore_asns = [6939, 174]  # ASNs to skip (HE, Cogent, etc.)

[bgpq4]
sources = ["ARIN", "RIPE", "AFRINIC", "APNIC", "LACNIC", "RADB", "ALTDB"]
```

That's it. No recompiling when you want to change the ignore list.

## Usage

```bash
# Validate configuration file
./frr-prefix-gen --validate

# Validate with strict mode (fail on warnings)
./frr-prefix-gen --validate --strict

# Health check - verifies bgpq4, vtysh, and PeeringDB API are accessible
./frr-prefix-gen --check

# Dry run - see what would change without applying anything
./frr-prefix-gen --dry-run

# Process all neighbors
./frr-prefix-gen

# Process specific ASN(s) only
./frr-prefix-gen --asn 13335 --asn 15169

# Enable timestamps in logs
./frr-prefix-gen --timestamps

# Use custom timestamp format
./frr-prefix-gen --timestamps --timestamp-format "%Y-%m-%d %H:%M:%S"

# Use custom config location
./frr-prefix-gen --config /path/to/config.toml
```

### Configuration Validation

The `--validate` flag checks your config file without requiring external services:

- **File validation**: Checks config exists and is readable
- **Syntax validation**: Ensures valid TOML
- **Value validation**: Verifies ranges (timeouts > 0, valid ASNs, etc.)
- **Warnings**: Flags suspicious values (debug logging, single IRR source, etc.)

Use `--strict` to fail on warnings as well as errors.

### Timestamps

Timestamps are disabled by default for human-readable format. Enable them with:

- **CLI**: `--timestamps` or `--timestamp-format "%F %T"`
- **Config**: Set `timestamps = true` in `[logging]` section

When enabled, human format logs look like:
```
[2026-03-07 18:30:00] INFO Starting FRR Prefix List Generator v0.1.0
```

JSON format always includes timestamps.

## Running on a schedule

### Systemd (Recommended)

For production deployments, use systemd with automatic scheduling. See the [Systemd Integration Guide](docs/systemd.md) for complete setup instructions including service units, timer configuration, and troubleshooting.

Benefits:
- Logs automatically go to journald (view with `journalctl -u frr-prefix-gen`)
- Automatic retry if system was down during scheduled time
- Built-in resource limits and security hardening
- View logs with timestamps: `journalctl -u frr-prefix-gen --output=short-iso`

### Crontab (Simple)

For simple setups, add to crontab:

```cron
0 0 * * * /usr/local/bin/frr-prefix-gen >> /var/log/prefix-gen.log 2>&1
```

Since it only modifies the running config, you can always roll back by reloading your saved FRR config if something goes wrong.

**Note**: When using cron, add `--timestamps` to include timestamps in log output.

## Building from source

```bash
git clone https://github.com/Team-Tritan/FRR-Prefix-List-Generator
cd FRR-Prefix-List-Generator
cargo build --release
# Binary will be at target/release/frr-prefix-gen
```

Needs Rust 1.85+ (for the 2024 edition).

## Contributing

PRs welcome. This was originally written in a few hours while I was at work, so there's definitely room for improvement.
