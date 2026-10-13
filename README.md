# glassmon

[![CI](https://github.com/7assanyahya/glassmon/actions/workflows/ci.yml/badge.svg)](https://github.com/7assanyahya/glassmon/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/glassmon.svg)](https://crates.io/crates/glassmon)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-orange.svg)](Cargo.toml)

> **A fast, good-looking terminal system monitor for macOS and Linux, featuring a Liquid Glass inspired aesthetic.**

```
 ╭── Overview ─── 2 Processes ─── 3 Network ─── 4 Disks ─── 5 Hardware ─── 6 Device ─────────── 14:42:01 ╮
 │                                                                                                       │
 │  CPU Usage [1]                                      Top Processes [2]                                 │
 │  Apple M3 Pro  32.4%                                PID     NAME             USER    CPU%   MEM       │
 │  ▅▆▇▃  ▂▅▆▇▃  ▂▅▆▇▃  ▂▅▆▇▃                          1984    rust-analyzer    hassan  14.5%  1.42 GB   │
 │  P0  ████████▍      42%   E0  █████▏        25%     240     WindowServer     _window 11.2%  680 MB    │
 │  P1  ███████████▌   58%   E1  ████▍         21%     3410    cargo            hassan  48.0%  890 MB    │
 │  P2  ███████        35%   E2  ██████        30%     4812    glassmon         hassan   1.8%   38 MB    │
 │                                                                                                       │
 │  Memory & Swap                                      Disks [4]               Sensors & Fans [5]        │
 │  RAM   18.4 GiB / 36.0 GiB (51.2%)                  Macintosh HD            CPU P-Cores   48°C        │
 │  Swap  120 MiB / 4.00 GiB (3.0%)                    ████████░░░░░░  44%     Apple SSD     35°C        │
 │  Press 22%                                          Throughput: 12.4 MB/s   Fans: 1450 RPM            │
 ╰───────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

---

## Highlights

- 💎 **Liquid Glass UI Approximation:** Rounded cards (`BorderType::Rounded`), sleek dark `#0b0d1a` background, vibrant accent tokens, eighth-block smooth gauge rendering (`▏▎▍▌▋▊▉█`), and responsive layouts.
- ⚡ **Zero Async, Lightweight:** Single-threaded loop driven by `crossterm::event::poll`. Uses `< 3%` of one CPU core at default 1-second ticks.
- 🍏 **First-Class macOS & Apple Silicon Support:** P/E core identification via `sysctl`, memory pressure monitoring, hardware profiling, and MacBook chassis port maps.
- 🐧 **Comprehensive Linux Support:** Direct parsing of `/sys/class/hwmon`, `/sys/class/power_supply`, `/proc/pressure` (PSI), and `/proc/diskstats`.
- 🛡️ **Zero Unsafe Code:** `#![forbid(unsafe_code)]` enforced crate-wide. Graceful signal handling (`SIGTERM`/`SIGKILL`) via `rustix`.
- 🔒 **100% Local & Private:** No network calls, telemetry, or remote tracking.
- 🎨 **Adaptive Themes:** Auto-detects 24-bit TrueColor (`COLORTERM`), smoothly degrades to nearest ANSI 256-color palette, supports `NO_COLOR` monochrome, and includes an accessible Light theme (`t`).
- 🎬 **Animated Demo Mode:** Run `glassmon --demo` on any machine to explore all 6 pages with deterministic animated waveforms without needing real sensors.

---

## Gallery

| Overview | Device Chassis |
| :---: | :---: |
| ![Overview](docs/img/overview.png) | ![Device](docs/img/device.png) |

| Processes | Disks Breakdown |
| :---: | :---: |
| ![Processes](docs/img/processes.png) | ![Disks](docs/img/disks.png) |

---

## Installation

### Cargo (crates.io)

```bash
cargo install glassmon
```

### Homebrew (macOS & Linux)

```bash
brew tap 7assanyahya/tap
brew install glassmon
```

### Pre-built Binaries

Download standalone archives from the [GitHub Releases](https://github.com/7assanyahya/glassmon/releases) page for:
- `x86_64-apple-darwin` (Intel macOS)
- `aarch64-apple-darwin` (Apple Silicon macOS)
- `x86_64-unknown-linux-gnu` (Linux x86_64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)

### Build From Source

```bash
git clone https://github.com/7assanyahya/glassmon.git
cd glassmon
cargo build --release
./target/release/glassmon
```

---

## Key Bindings

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Switch to next / previous page |
| `1` - `6` | Direct jump: **1** Overview, **2** Processes, **3** Network, **4** Disks, **5** Hardware, **6** Device |
| `t` | Toggle Dark / Light theme |
| `p` | Pause / resume real-time sampling |
| `?` | Open interactive help overlay |
| `q` / `Ctrl+C` | Quit |
| `j` / `k` / `↑` / `↓` | Move row selection in lists |
| `g` / `G` | Jump to start / end of list |
| `PgUp` / `PgDn` | Scroll page up / down |
| `s` | Cycle sort column (Processes: CPU, MEM, PID, Name; Disks: Size, Mount) |
| `r` | Reverse sort order (Processes) / Rescan hardware & ports (Hardware, Device) |
| `/` | Open inline case-insensitive search filter |
| `Esc` | Clear filter / dismiss modal dialog |
| `K` | Open terminate process modal (**SIGTERM**) |
| `Shift+K` | In terminate modal: escalate to force kill (**SIGKILL**) |
| `n` | Cycle selected network interface graph |
| `h` / `l` | Switch active storage volume in breakdown card |
| `e` | Open unmount / eject volume modal |
| `u` | Toggle temperature scale (°C / °F) |
| `1` - `7` | Select chassis port on Device page |
| Mouse | Click tabs, scroll tables, click rows (toggle with `--no-mouse`) |

---

## Platform Support Matrix

`glassmon` handles platform variations gracefully. Any metric unavailable on your hardware is represented as `n/a` in a secondary color rather than hiding the card:

| Feature / Metric | macOS (Apple Silicon) | macOS (Intel) | Linux (x86_64 / ARM64) |
| :--- | :---: | :---: | :---: |
| **CPU Usage & Frequency** | ✅ | ✅ | ✅ |
| **P-Core / E-Core Split** | ✅ (`sysctl`) | `n/a` | `n/a` |
| **Memory & Swap** | ✅ | ✅ | ✅ |
| **Memory Pressure** | ✅ (`kern.memorystatus`) | ✅ | ✅ (`/proc/pressure`) |
| **Per-Process CPU & Memory** | ✅ | ✅ | ✅ |
| **Process Signal Control** | ✅ (`rustix`) | ✅ (`rustix`) | ✅ (`rustix`) |
| **Per-Interface Rates** | ✅ | ✅ | ✅ |
| **Storage Category Breakdown** | ✅ | ✅ | `n/a` (shows Dir split) |
| **I/O Rates & Latency** | ✅ | ✅ | ✅ (`/proc/diskstats`) |
| **SMART Drive Health** | ✅ (`system_profiler`) | ✅ | ✅ (`smartctl`) |
| **Thermals & Fans** | ✅ | ✅ | ✅ (`/sys/class/hwmon`) |
| **Battery Health & Cycles** | ✅ | ✅ | ✅ (`/sys/class/power_supply`) |
| **Wi-Fi Link & Signal (dBm)** | ✅ | ✅ | ✅ (`iw` / `/sys/class/net`) |
| **Chassis Port Visualization** | ✅ (MacBook models) | ✅ (Generic) | ✅ (Generic) |

---

## Configuration

`glassmon` creates an optional configuration file at `~/.config/glassmon/config.toml`:

```toml
# Sampling interval in milliseconds
interval_ms = 1000

# Color theme: "dark" or "light"
theme = "dark"

# Temperature scale: "celsius" or "fahrenheit"
temp_unit = "celsius"

# Enable mouse capture (clicking tabs, scrolling lists)
mouse = true

# Default process sorting column: "cpu", "mem", "pid", "name"
process_sort = "cpu"
```

Command-line flags override configuration values when specified:
```bash
glassmon --interval 500 --theme light --no-mouse
```

---

## Privacy Policy

`glassmon` is completely local:
- No network requests or telemetry transmissions.
- No third-party data tracking.
- Reads only local system metrics and logs.

See [`SECURITY.md`](SECURITY.md) for full details and private vulnerability reporting instructions.

---

## Contributing

Contributions are welcome! Please check out [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup guides, how to add port maps for new machine models, and PR guidelines.

---

## License

Dual-licensed under either of:
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))

at your option.
