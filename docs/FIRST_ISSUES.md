# Good First Issues

Here are 10 concrete, well-scoped issues ready to be pasted directly into GitHub Issues for new open-source contributors:

---

### Issue 1: Add port chassis layout for MacBook Air 15" (M2 / M3)
- **Labels:** `good first issue`, `enhancement`, `device`
- **Description:**
  Add port definitions for the 15-inch MacBook Air (`Mac14,15`, `Mac15,13`) to `src/data/macos.rs` using the layout instructions in `docs/ADDING_A_MACHINE.md`.
  - Left edge: MagSafe 3, 2x Thunderbolt / USB 4 ports.
  - Right edge: 3.5mm Headphone Jack.
- **Acceptance Criteria:**
  - Running on a 15" MacBook Air displays correct port names and chassis diagram on Page 6 (Device).
  - Unit tests updated with model strings.

---

### Issue 2: Add port layout for Mac mini (M2 / M2 Pro / M4)
- **Labels:** `good first issue`, `enhancement`, `device`
- **Description:**
  Add a desktop port mapping for Mac mini models (`Mac14,3`, `Mac14,12`).
  - Back ports: Ethernet (Gigabit / 10Gb), 2x or 4x Thunderbolt 4, HDMI, 2x USB-A, 3.5mm Headphone Jack.
- **Acceptance Criteria:**
  - Mac mini models are identified as `DeviceForm::Desktop`.
  - Ports map accurately to the rear I/O layout.

---

### Issue 3: Add "Nord" color theme
- **Labels:** `good first issue`, `ui`, `theme`
- **Description:**
  Add a new theme preset based on the popular **Nord** palette (`#2e3440` polar night background, `#3b4252` cards, `#88c0d0` frost blue, `#a3be8c` aurora green).
- **Acceptance Criteria:**
  - Add `ThemeMode::Nord` in `src/ui/theme.rs`.
  - Cycle through Dark -> Light -> Nord when pressing `t`.
  - Color contrast ratio for text on card backgrounds exceeds 4.5:1.

---

### Issue 4: Export process snapshot to CSV / JSON
- **Labels:** `good first issue`, `feature`, `cli`
- **Description:**
  Add a `--dump <csv|json>` CLI flag that samples system processes once, writes the formatted output to stdout, and exits without launching the full TUI.
- **Acceptance Criteria:**
  - `glassmon --dump json` prints valid JSON containing CPU, memory, and process list.
  - Useful for shell scripts and headless monitoring pipelines.

---

### Issue 5: Add battery charge threshold / limit indicator
- **Labels:** `good first issue`, `hardware`, `battery`
- **Description:**
  On macOS laptops using "Optimized Battery Charging" or tools like AlDente, battery charging pauses at 80%.
  Detect this state in `src/data/macos.rs` and render an `[80% Limit]` chip next to the battery charge bar.
- **Acceptance Criteria:**
  - Battery card displays charging limit when AC is connected but charging is paused.

---

### Issue 6: Add configurable process refresh interval in config.toml
- **Labels:** `good first issue`, `config`, `optimization`
- **Description:**
  Allow users to configure independent refresh cadences (e.g. 500ms for CPU, 2000ms for processes) in `~/.config/glassmon/config.toml` to further minimize background overhead on resource-constrained systems.
- **Acceptance Criteria:**
  - `config.rs` deserializes `process_interval_ms`.
  - Collector respects the independent rate.

---

### Issue 7: Add Windows platform stub collector
- **Labels:** `good first issue`, `cross-platform`, `collector`
- **Description:**
  Add a `src/data/windows.rs` module gated behind `#[cfg(target_os = "windows")]` implementing the `Collector` trait using `sysinfo` with graceful `n/a` fallbacks for macOS/Linux specific fields.
- **Acceptance Criteria:**
  - `cargo check --target x86_64-pc-windows-msvc` compiles without errors.

---

### Issue 8: Add search filter history in Processes view
- **Labels:** `good first issue`, `ui`, `processes`
- **Description:**
  Allow users to press `Up` / `Down` while in search mode (`/`) to recall recently used process filter strings.
- **Acceptance Criteria:**
  - Store previous 5 queries in `App::process_filter_history`.
  - Navigating with arrows populates the query input.

---

### Issue 9: Custom disk category color configuration
- **Labels:** `good first issue`, `config`, `disks`
- **Description:**
  In the Storage Breakdown card on the Disks page, categories (Apps, Developer, Media, Documents, System) use hardcoded colors. Allow customizing segment colors in `config.toml`.
- **Acceptance Criteria:**
  - Category hex colors can be overridden in `[theme.storage]` section of `config.toml`.

---

### Issue 10: Add copy selected PID to system clipboard
- **Labels:** `good first issue`, `ui`, `feature`
- **Description:**
  Pressing `y` on the Processes page should copy the selected process's PID to the system clipboard (via OSC 52 terminal clipboard escape sequence or `copypasta`), displaying a "Copied PID 1234 to clipboard" toast notification.
- **Acceptance Criteria:**
  - Pressing `y` triggers a clipboard write and toast popup.
  - Works without requiring native clipboard dependencies by utilizing terminal OSC 52 sequences where supported.
