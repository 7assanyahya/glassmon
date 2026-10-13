# Contributing to glassmon

Thank you for your interest in contributing to `glassmon`! This guide explains how to set up your environment, develop new features, add hardware models, and submit pull requests.

---

## Development Setup

1. **Prerequisites:**
   - Rust stable (`1.88+`)
   - `git`
   - Terminal with TrueColor support recommended (e.g. Ghostty, iTerm2, Alacritty, Kitty)

2. **Clone & Run:**
   ```bash
   git clone https://github.com/7assanyahya/glassmon.git
   cd glassmon
   cargo run -- --demo
   ```

   The `--demo` flag starts the application with simulated animated data, allowing you to develop and verify all UI components and pages on any operating system without hardware dependencies.

---

## Project Layout

```
src/
├── main.rs            # CLI argument parsing, terminal initialization, event loop
├── lib.rs             # Crate root exporting modules for tests & library usage
├── app.rs             # Application state (pages, active selections, filters, modals, toasts)
├── config.rs          # TOML configuration loading and persistence
├── model.rs           # Pure Snapshot data contract (UI only reads this)
├── units.rs           # Formatting helpers (bytes, rates, durations, temps)
├── tui.rs             # Terminal raw mode, alt screen, panic hook & clean restore
├── data/
│   ├── mod.rs         # Collector trait definition
│   ├── collector.rs   # Common sysinfo collection, ring buffers, delta rate math
│   ├── macos.rs       # macOS sysctl, system_profiler parsing, starship-battery
│   ├── linux.rs       # Linux /sys/class/hwmon, /sys/class/power_supply, PSI
│   └── demo.rs        # Deterministic animated mock collector
└── ui/
    ├── mod.rs         # Top-level draw routine, header shell, footer, breakpoints
    ├── theme.rs       # Palette definitions, TrueColor / 256 / monochrome mapping
    ├── widgets/       # Rounded Card, SmoothBar, StackedBar, Pill, Keycap, Modal, Toast
    └── pages/         # Page renderers (Overview, Processes, Network, Disks, Hardware, Device)
```

---

## How to Add a New Page

1. **Define the Page Enum:**
   Add a new variant in `src/app.rs` (`pub enum Page`), with its `title()`, `subtitle()`, and `needs()`.
2. **Implement the Renderer:**
   Create `src/ui/pages/<name>.rs` implementing:
   ```rust
   pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) { ... }
   ```
3. **Register the Page:**
   Export it in `src/ui/pages/mod.rs` and add it to `render_page` in `src/ui/mod.rs` and `TabBar` in `src/ui/widgets/tab_bar.rs`.
4. **Snapshot Test:**
   Add a test case in `tests/page_snapshots.rs` using `TestBackend` at 120x36 and 80x24.

---

## How to Add a Platform Collector

1. Extend `Snapshot` in `src/model.rs` if your platform exposes new metrics (always use `Option<T>` for anything platform-specific).
2. Implement the `Collector` trait in `src/data/<platform>.rs`:
   ```rust
   impl Collector for MyPlatformCollector {
       fn sample(&mut self, needs: &Needs) -> Snapshot { ... }
   }
   ```
3. Wire the collector conditionally in `src/main.rs`.

---

## How to Add a Port Layout for a New Machine Model

See [`docs/ADDING_A_MACHINE.md`](docs/ADDING_A_MACHINE.md) for detailed instructions on adding port definitions for new laptop or desktop models.

---

## Testing & Quality Assurance

Before submitting a pull request, run the following commands locally:

```bash
# 1. Format check
cargo fmt --check

# 2. Clippy check (must pass with zero warnings)
cargo clippy --all-targets -- -D warnings

# 3. Unit and snapshot integration tests
cargo test

# 4. Dependency license and vulnerability check
cargo deny check
cargo audit
```

---

## Commit Guidelines

We use **Conventional Commits**:
- `feat:` New features or UI pages
- `fix:` Bug fixes or layout corrections
- `docs:` Documentation improvements
- `ci:` CI workflow updates
- `chore:` Maintenance, refactoring, dependency bumps

Example:
```
feat(device): add port chassis mapping for MacBook Air M3
```

---

## Capturing UI Screenshots

To capture clean screenshots for documentation:
1. Run `cargo run -- --demo`.
2. Resize terminal to `120x36` or larger.
3. Switch pages using `1`-`6` or toggle theme with `t`.
4. Save images to `docs/img/<page_name>.png`.

---

## PR Checklist

- [ ] `cargo fmt --check` passes without diffs.
- [ ] `cargo clippy --all-targets -- -D warnings` reports 0 warnings.
- [ ] `cargo test` passes all unit and snapshot tests.
- [ ] No `unsafe` blocks added (`#![forbid(unsafe_code)]` remains strictly enforced).
- [ ] Added or updated tests in `tests/` where applicable.
