# ASUS Fan Control in Rust

A small daemon for Linux ASUS laptops that toggles fan mode based on CPU temperature.

It was built for machines where common fan control paths are missing, but `pwm1_enable`
is available under `asus-nb-wmi`.

## What it does

- Reads CPU package temperature from hardware sensors
- Uses smoothing + hysteresis to avoid frequent fan mode switching
- Switches `pwm1_enable` to:
  - `0` → full speed
  - `2` → automatic mode
- Exposes a Unix socket so you can query status with `fanctl status`

## Requirements

- Linux with ASUS `asus-nb-wmi` hwmon support
- A detectable CPU package sensor (`Package ...` label)
- Rust toolchain (`rustup`)
- `systemd`
- Root privileges (service runs as root and writes to `/sys/.../pwm1_enable`)

## Install

From the repository root:

```bash
make install-all
```

This will build the binary, install it to `/usr/local/bin/fanctl`, install `fanctl.service`,
reload systemd, and start the service.

## Usage

Check live controller status:

```bash
fanctl status
```

Check service state:

```bash
make status-systemd
```

Stop service:

```bash
make stop-systemd
```

## Optional configuration

If present, the daemon loads config from:

- `/etc/fanctl/config.toml`

Missing values fall back to defaults.

Supported fields:

- `t_enable` (alias: `threshold_enable`) — default `70.0`
- `t_auto` (alias: `threshold_auto`) — default `60.0`
- `b_rise` (alias: `bias_rise`) — default `0.6`
- `b_drop` (alias: `bias_drop`) — default `0.4`

Example:

```toml
t_enable = 72.0
t_auto = 62.0
b_rise = 0.7
b_drop = 0.3
```

## Uninstall service

```bash
make uninstall-systemd
```

## Notes

This project is hardware-specific and may not work on other laptop models or sensor
layouts.
