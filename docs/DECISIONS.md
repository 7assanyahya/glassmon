# Architecture & Design Decisions

This document records key decisions made during the design and implementation of `glassmon`.

## 1. Project Name: `glassmon`

- **Context:** The name `sysmon` is already registered on crates.io (v0.2.5).
- **Decision:** Selected `glassmon` ("Glass Monitor"). It directly reflects the Liquid Glass visual theme, is short, memorable, and verified free on crates.io (HTTP 404 on API check).

## 2. Edition & MSRV (Minimum Supported Rust Version)

- **Context:** `Cargo.toml` specifies MSRV for predictability across build environments and CI.
- **Decision:** Set MSRV to Rust `1.88`. Edition `2024` with Rust 1.89+ toolchains, falling back cleanly. Dependencies pinned to stable releases supporting 1.88 (`sysinfo` 0.38.4, `ratatui` 0.30.2, `starship-battery` 0.10.2).

## 3. Strict `#![forbid(unsafe_code)]`

- **Context:** Safety is paramount for a monitoring tool running with high system privilege or in everyday terminal workflows.
- **Decision:** No `unsafe` in our codebase (`#![forbid(unsafe_code)]`). Where OS signals (SIGTERM/SIGKILL) or sysctls are needed, we rely on safe standard libraries or well-audited bindings like `rustix` or safe `libc`/`mach2` abstractions provided by dependencies.

## 4. Single-Threaded Event Loop & Sampling Architecture

- **Context:** Must be lightweight, using < 3% of a single CPU core at 1-second refresh intervals, without async runtime bloat.
- **Decision:** Single thread with `crossterm::event::poll(tick_remaining)`.
- **Optimization:** Page-specific `Needs` tracking: only scan the process table, read detailed hardware profiles, or query sensor trees if the currently active page actually requires them.
- Slow-path hardware data (like `system_profiler` on macOS) is cached and refreshed on a slow timer (30s) or on manual rescan (`r`).

## 5. Separation of Concerns: Collector -> Snapshot -> UI

- **Context:** Multiple target OS platforms (macOS Apple Silicon & Intel, Linux x86_64 & aarch64) and mock/demo mode.
- **Decision:**
  - `model::Snapshot` is a pure, immutable snapshot of system state with `Option<T>` for anything potentially missing on a platform.
  - UI widgets and page renderers only read `&Snapshot`, never touching system APIs directly.
  - Collector trait: `sample(&mut self, needs: &Needs) -> Snapshot`.
  - `--demo` provides deterministic animated waveforms matching the mockups without querying hardware.

## 6. Color Fallbacks & Themes

- **Context:** Terminal capabilities vary widely (truecolor 24-bit, 256-color, 16-color, or `NO_COLOR`).
- **Decision:**
  - Detection checks `COLORTERM == "truecolor"` / `"24bit"` or explicit configuration.
  - When truecolor is unavailable, RGB colors are mapped to the nearest xterm 256-color palette.
  - When `NO_COLOR` is present or monochrome mode is requested, styles use simple bold/dim/reverse video without color codes.
  - Both Dark ("Liquid Glass" deep blue/slate) and Light (high-contrast > 4.5:1) themes supported, toggled with `t`.

## 7. Machine Model & Port Layouts

- **Context:** Visual port layout on the Device page.
- **Decision:** Data-driven port definitions table for common Mac models (MacBook Pro 14"/16", MacBook Air, Mac mini, Mac Studio) and a graceful generic fallback for unknown machines and Linux.
