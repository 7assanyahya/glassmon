# Changelog

All notable changes to `glassmon` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-13

### Added
- **Liquid Glass Visual Aesthetic:** Dark (`#0b0d1a` background, rounded cards, vibrant accents) and high-contrast Light themes.
- **Adaptive Terminal Capabilities:** 24-bit TrueColor auto-detection with fallback to 256-color palette and `NO_COLOR` monochrome support.
- **Six Specialized Monitoring Pages:**
  - **Overview (120x36):** CPU history & per-core grid, RAM/Swap/Pressure bars, Network sparklines, top processes, physical disks and thermals.
  - **Processes:** Full process table with CPU, MEM, Threads, State, Command, sort cycling (`s`), direction toggle (`r`), case-insensitive inline filter (`/`), and confirmed signal sending (`K` for SIGTERM, `Shift+K` for SIGKILL).
  - **Network:** Per-interface RX/TX rates, cumulative traffic, 60-sample scrolling sparklines, and interface switching (`n`).
  - **Disks:** Category storage breakdown (stacked bar with largest-remainder width allocation), I/O throughput with IOPS/latency tiles, physical volume mounts, SMART metrics, and volume unmounting (`e`).
  - **Hardware:** Machine specs, Apple Silicon P/E core identification, battery diagnostics, thermals with °C/°F toggle (`u`), fan RPM bars, Wi-Fi link parameters, Bluetooth peripherals, and storage devices.
  - **Device:** Chassis layout visualization with numbered port badges, link speeds, 2x2 health tiles, and port inspector (`1`-`7`).
- **Deterministic Demo Mode:** `--demo` flag for testing and showcasing full UI workflows with realistic animated mock data without sensor requirements.
- **Zero-Unsafe Core:** Verified `#![forbid(unsafe_code)]` with robust signal dispatch via `rustix`.
- **Clean Terminal Recovery:** Automated terminal cleanup on normal exit, signals, and panics.
- **Configuration:** Optional persistence in `~/.config/glassmon/config.toml`.
