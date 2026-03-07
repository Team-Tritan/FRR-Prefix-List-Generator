# Systemd Integration

This guide explains how to run FRR Prefix List Generator as a systemd service with automatic scheduling using systemd timers.

## Quick Start

Copy these files to your systemd configuration:

### 1. Service Unit

Create `/etc/systemd/system/frr-prefix-gen.service`:

```ini
[Unit]
Description=FRR Prefix List Generator
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log

[Install]
WantedBy=multi-user.target
```

### 2. Timer Unit

Create `/etc/systemd/system/frr-prefix-gen.timer`:

```ini
[Unit]
Description=Run FRR Prefix List Generator every 6 hours

[Timer]
OnBootSec=5min
OnUnitActiveSec=6h
Persistent=true

[Install]
WantedBy=timers.target
```

### 3. Enable and Start

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable the timer (starts automatically on boot)
sudo systemctl enable frr-prefix-gen.timer

# Start the timer now
sudo systemctl start frr-prefix-gen.timer

# Check status
sudo systemctl status frr-prefix-gen.timer
```

## Configuration Options Explained

### Service Unit Breakdown

```ini
[Unit]
Description=FRR Prefix List Generator
After=network.target
```
- **Description**: What appears in systemctl status and logs
- **After**: Ensures network is up before running (needed for PeeringDB API)

```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen
StandardOutput=journal
StandardError=journal
```
- **Type=oneshot**: Service runs once and exits (perfect for this tool)
- **ExecStart**: Path to your compiled binary
- **StandardOutput/Error=journal**: Send logs to systemd journal

```ini
# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log
```
- Security features to limit what the service can access
- **Note**: If using custom config location, add it to ReadWritePaths

```ini
[Install]
WantedBy=multi-user.target
```
- Makes service start as part of normal system boot

### Timer Unit Breakdown

```ini
[Timer]
OnBootSec=5min
OnUnitActiveSec=6h
Persistent=true
```
- **OnBootSec=5min**: Run 5 minutes after boot
- **OnUnitActiveSec=6h**: Run every 6 hours after last execution
- **Persistent=true**: If system was off during scheduled time, run immediately on boot

## Customization Templates

### Custom Config Location

```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen --config /etc/my-config.toml
StandardOutput=journal
StandardError=journal
ReadWritePaths=/etc/my-config.toml /var/log
```

### Dry Run Mode (for testing)

```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen --dry-run
StandardOutput=journal
StandardError=journal
```

### Custom Run Frequency

**Every hour:**
```ini
[Timer]
OnBootSec=1min
OnUnitActiveSec=1h
Persistent=true
```

**Daily at 2 AM:**
```ini
[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true
```

**Weekly:**
```ini
[Timer]
OnCalendar=weekly
Persistent=true
```

### With Timestamps Enabled

```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen --timestamps
StandardOutput=journal
StandardError=journal
```

Or use config file:
```toml
[logging]
timestamps = true
timestamp_format = "%Y-%m-%d %H:%M:%S"
```

### Running as Non-Root User

```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen
User=frr
Group=frr
# Add vtysh to user's PATH or use full path
ExecStartPre=/bin/bash -c 'which vtysh'
StandardOutput=journal
StandardError=journal
```

**Note**: The user needs permission to run `vtysh` and access FRR sockets.

## Viewing Logs

### Basic Log Viewing

```bash
# View all logs for the service
sudo journalctl -u frr-prefix-gen

# Follow logs in real-time
sudo journalctl -u frr-prefix-gen -f

# View last 50 lines
sudo journalctl -u frr-prefix-gen -n 50

# View logs since last boot
sudo journalctl -u frr-prefix-gen -b
```

### With Timestamps

```bash
# Show timestamps in your local timezone
sudo journalctl -u frr-prefix-gen --output=short

# Show full timestamps (ISO 8601)
sudo journalctl -u frr-prefix-gen --output=short-iso

# Show precise timestamps (with microseconds)
sudo journalctl -u frr-prefix-gen --output=short-precise
```

### Filtering Logs

```bash
# Logs from last hour
sudo journalctl -u frr-prefix-gen --since "1 hour ago"

# Logs from specific time
sudo journalctl -u frr-prefix-gen --since "2026-03-07 10:00:00" --until "2026-03-07 14:00:00"

# Only errors
sudo journalctl -u frr-prefix-gen --priority=err

# Errors and warnings
sudo journalctl -u frr-prefix-gen --priority=warning
```

## Troubleshooting

### Service Failing to Start

```bash
# Check detailed status
sudo systemctl status frr-prefix-gen.service

# View full logs
sudo journalctl -u frr-prefix-gen.service -n 100 --no-pager

# Test manually
sudo /usr/local/bin/frr-prefix-gen --check
```

### Permission Issues

If you see "Permission denied" errors:

```bash
# Check binary permissions
ls -la /usr/local/bin/frr-prefix-gen

# Fix ownership (if needed)
sudo chown root:root /usr/local/bin/frr-prefix-gen
sudo chmod 755 /usr/local/bin/frr-prefix-gen

# Check vtysh is accessible
sudo which vtysh
sudo vtysh -c "show version"
```

### Timer Not Running

```bash
# Check if timer is enabled
sudo systemctl is-enabled frr-prefix-gen.timer

# List all timers
sudo systemctl list-timers --all

# Check for errors
sudo systemctl status frr-prefix-gen.timer
```

### Manual Trigger

Force a run immediately (useful for testing):

```bash
sudo systemctl start frr-prefix-gen.service
```

## Complete Production Example

Here's a complete production-ready configuration with all features:

**frr-prefix-gen.service:**
```ini
[Unit]
Description=FRR Prefix List Generator
After=network.target
Wants=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/frr-prefix-gen --timestamps
StandardOutput=journal
StandardError=journal
SyslogIdentifier=frr-prefix-gen

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/etc/frr-prefix-gen /var/log

# Resource limits
MemoryMax=100M
CPUQuota=50%
TimeoutStartSec=300

[Install]
WantedBy=multi-user.target
```

**frr-prefix-gen.timer:**
```ini
[Unit]
Description=Run FRR Prefix List Generator every 6 hours
Documentation=https://github.com/Team-Tritan/FRR-Prefix-List-Generator

[Timer]
OnBootSec=5min
OnUnitActiveSec=6h
RandomizedDelaySec=300
Persistent=true

[Install]
WantedBy=timers.target
```

**config.toml:**
```toml
[general]
concurrency = 4

[logging]
level = "info"
format = "human"
timestamps = true
timestamp_format = "%Y-%m-%d %H:%M:%S"

[filter]
ignore_asns = []

[bgpq4]
sources = ["ARIN", "RIPE", "AFRINIC", "APNIC", "LACNIC", "RADB", "ALTDB"]
```

## Uninstall

```bash
# Stop and disable
sudo systemctl stop frr-prefix-gen.timer
sudo systemctl disable frr-prefix-gen.timer
sudo systemctl stop frr-prefix-gen.service
sudo systemctl disable frr-prefix-gen.service

# Remove files
sudo rm /etc/systemd/system/frr-prefix-gen.service
sudo rm /etc/systemd/system/frr-prefix-gen.timer

# Reload systemd
sudo systemctl daemon-reload
```
