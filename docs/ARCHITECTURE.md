# glassmon Architecture

`glassmon` is designed around strict separation of data collection and terminal UI presentation. The terminal interface never interacts with the operating system directly; it only reads an immutable `Snapshot`.

---

## High-Level Data Flow

```text
┌────────────────────────────────────────────────────────┐
│                   Operating System                     │
│  (sysinfo, sysctl, /sys, /proc, system_profiler)       │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                   Collector Engine                     │
│  - SystemCollector (sysinfo metrics & delta rates)     │
│  - MacOsCollector (Apple Silicon P/E, profiler, power) │
│  - LinuxCollector (/sys/hwmon, power_supply, PSI)      │
│  - DemoCollector (Deterministic animated simulation)   │
└───────────────────────────┬────────────────────────────┘
                            │
             Produces immutable Snapshot (1/tick)
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                 model::Snapshot State                  │
│  • CPU (History, Cores, P/E label)                     │
│  • Memory (RAM, Swap, Pressure %)                      │
│  • Disks (Volumes, I/O rates, SMART, Breakdown)        │
│  • Network (Interfaces, RX/TX histories)               │
│  • Processes (PID, CPU, MEM, threads, state, cmd)      │
│  • Hardware & Ports (Chassis matrix, link status)      │
│  • Sensors (Thermals, Fans, Battery draw)              │
└───────────────────────────┬────────────────────────────┘
                            │
              UI renders Snapshot + App state
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│                      UI Engine                         │
│  - Theme Token System (TrueColor, 256-color, NO_COLOR) │
│  - Widgets (Cards, SmoothBar, StackedBar, Modal)       │
│  - Pages (Overview, Procs, Net, Disks, HW, Device)     │
│  - Terminal Render Buffer (Ratatui CrosstermBackend)   │
└────────────────────────────────────────────────────────┘
```

---

## Core Principles

### 1. Single-Threaded Polling Loop

Instead of running background async threads or multiple channels that contend for locks, `glassmon` executes on a single thread:
- `crossterm::event::poll(remaining_tick_timeout)` waits for user input.
- If input arrives, the UI handles keystrokes or mouse clicks immediately.
- When the timer expires, the active collector runs a sample pass.
- This design eliminates runtime overhead, async dependencies, and multithreading locks, maintaining CPU usage below 3% of a single core.

### 2. Page-Specific `Needs` Optimization

System calls like scanning hundreds of processes, reading full directory breakdown trees, or querying `system_profiler` can consume CPU if run every second.
The `Needs` struct tells the collector what data the current page requires:
- When viewing the **Network** page, process scans are skipped entirely.
- When viewing **Overview**, detailed per-process thread inspections are omitted.
- Slow-path hardware data (e.g. `system_profiler`) is cached and only refreshed on a 30-second cadence or upon pressing `r`.

### 3. Graceful Metric Degradation (`Option<T>`)

Hardware differs across architectures:
- Apple Silicon exposes P-cores and E-cores; Intel Mac and Linux systems report standard cores.
- Desktops have no battery; passively cooled machines have no fans.
- Linux servers may lack Wi-Fi interfaces.

Every platform-specific sensor is stored as an `Option<T>` in `Snapshot`. Widgets and table rows render `n/a` in a subtle secondary color when a value is `None`, rather than hiding containers or panicking.

### 4. Zero Unsafe Code

`glassmon` forbids unsafe blocks across the entire crate (`#![forbid(unsafe_code)]`). Terminal raw-mode recovery and process signal dispatching use safe, audited abstractions like `rustix`.
