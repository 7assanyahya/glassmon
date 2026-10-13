//! glassmon: terminal system monitor entrypoint.

use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use glassmon::app::{App, Page, ProcessSort};
use glassmon::config::Config;
use glassmon::data::Collector;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use glassmon::data::collector::SystemCollector;
use glassmon::data::demo::DemoCollector;
#[cfg(target_os = "linux")]
use glassmon::data::linux::LinuxCollector;
#[cfg(target_os = "macos")]
use glassmon::data::macos::MacOsCollector;
use glassmon::model::Snapshot;
use glassmon::ui::theme::{ColorCapability, ThemeMode};
use glassmon::ui::widgets::modal::ModalType;
use glassmon::ui::widgets::toast::ToastKind;

#[derive(Parser, Debug)]
#[command(
    name = "glassmon",
    about = "A fast, good-looking terminal system monitor for macOS and Linux with Liquid Glass aesthetic",
    version
)]
struct Args {
    /// Run with deterministic animated demo data (no real sensors required)
    #[arg(long)]
    demo: bool,

    /// Disable mouse interaction
    #[arg(long)]
    no_mouse: bool,

    /// Sampling update interval in milliseconds
    #[arg(short, long)]
    interval: Option<u64>,

    /// Override visual theme (dark or light)
    #[arg(long)]
    theme: Option<String>,

    /// Force monochrome mode (no ANSI colors)
    #[arg(long)]
    no_color: bool,

    /// Verbose diagnostic logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load();

    let mouse_enabled = !args.no_mouse && config.mouse;
    let (mut terminal, _guard) = glassmon::tui::init(mouse_enabled)?;

    // Determine color capability and theme
    let color_cap = if args.no_color {
        ColorCapability::Monochrome
    } else {
        ColorCapability::detect()
    };

    let theme_mode = match args.theme.as_deref().unwrap_or(&config.theme) {
        "light" => ThemeMode::Light,
        _ => ThemeMode::Dark,
    };

    let mut app = App::new(theme_mode, color_cap);
    if config.temp_unit == "fahrenheit" {
        app.fahrenheit = true;
    }

    match config.process_sort.as_str() {
        "mem" => app.process_sort = ProcessSort::Memory,
        "pid" => app.process_sort = ProcessSort::Pid,
        "name" => app.process_sort = ProcessSort::Name,
        _ => app.process_sort = ProcessSort::Cpu,
    }

    // Initialize collector
    let mut collector: Box<dyn Collector> = if args.demo {
        Box::new(DemoCollector::new())
    } else {
        #[cfg(target_os = "macos")]
        {
            Box::new(MacOsCollector::new())
        }
        #[cfg(target_os = "linux")]
        {
            Box::new(LinuxCollector::new())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Box::new(SystemCollector::new())
        }
    };

    let interval = Duration::from_millis(args.interval.unwrap_or(config.interval_ms).max(100));
    let mut last_tick = Instant::now();
    let mut current_snapshot = collector.sample(&app.page.needs());

    // Main application event loop
    while !app.should_quit {
        // Draw current frame
        terminal.draw(|f| {
            glassmon::ui::draw(f, &app, &current_snapshot);
        })?;

        // Calculate time to wait until next tick
        let timeout = interval
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(10));

        // Poll for user events
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key_event(
                        &mut app,
                        key.code,
                        key.modifiers,
                        &mut current_snapshot,
                        &mut *collector,
                    );
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(&mut app, mouse.kind, mouse.column, mouse.row);
                }
                Event::Resize(_, _) => {
                    // Terminal automatically resizes buffer on next draw
                }
                _ => {}
            }
        }

        // Sampling tick (if not paused and interval elapsed)
        if last_tick.elapsed() >= interval {
            if !app.paused {
                if app.request_rescan {
                    collector.rescan_hardware();
                    app.request_rescan = false;
                }
                current_snapshot = collector.sample(&app.page.needs());
            }
            app.clean_expired_toasts();
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn handle_key_event(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    snapshot: &mut Snapshot,
    collector: &mut dyn Collector,
) {
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
    let shift = modifiers.contains(KeyModifiers::SHIFT);

    // Global Quit shortcuts: 'q' (outside filter mode) or Ctrl+C
    if (code == KeyCode::Char('c') && ctrl)
        || (!app.filter_active && app.active_modal.is_none() && code == KeyCode::Char('q'))
    {
        app.should_quit = true;
        return;
    }

    // Modal active handling
    if let Some(modal) = &mut app.active_modal {
        match code {
            KeyCode::Esc => {
                app.active_modal = None;
            }
            KeyCode::Char('k') | KeyCode::Char('K') if shift => {
                if let ModalType::ConfirmKill {
                    escalate_sigkill, ..
                } = modal
                {
                    *escalate_sigkill = true;
                    app.add_toast("Escalated signal to SIGKILL", ToastKind::Warning);
                }
            }
            KeyCode::Enter => match modal {
                ModalType::ConfirmKill {
                    pid,
                    name,
                    escalate_sigkill,
                } => {
                    let target_pid = *pid;
                    let proc_name = name.clone();
                    let use_kill = *escalate_sigkill;
                    app.active_modal = None;
                    dispatch_kill(app, target_pid, &proc_name, use_kill);
                }
                ModalType::ConfirmEject { mount, device } => {
                    let m = mount.clone();
                    let d = device.clone();
                    app.active_modal = None;
                    dispatch_eject(app, &m, &d);
                }
                ModalType::Help => {
                    app.active_modal = None;
                }
            },
            _ => {}
        }
        return;
    }

    // Filter mode active handling (Processes page inline search)
    if app.filter_active {
        match code {
            KeyCode::Esc => {
                app.filter_active = false;
                app.process_filter.clear();
            }
            KeyCode::Enter => {
                app.filter_active = false;
            }
            KeyCode::Backspace => {
                app.process_filter.pop();
            }
            KeyCode::Char(c) => {
                app.process_filter.push(c);
            }
            _ => {}
        }
        return;
    }

    // Normal navigation & controls
    match code {
        // Page switching
        KeyCode::Tab => {
            app.page = if shift {
                app.page.prev()
            } else {
                app.page.next()
            };
        }
        KeyCode::BackTab => {
            app.page = app.page.prev();
        }
        KeyCode::Char('1') => app.page = Page::Overview,
        KeyCode::Char('2') => app.page = Page::Processes,
        KeyCode::Char('3') => app.page = Page::Network,
        KeyCode::Char('4') => app.page = Page::Disks,
        KeyCode::Char('5') => app.page = Page::Hardware,
        KeyCode::Char('6') => app.page = Page::Device,

        // Global shortcuts
        KeyCode::Char('t') => app.toggle_theme(),
        KeyCode::Char('p') => app.toggle_pause(),
        KeyCode::Char('?') => app.active_modal = Some(ModalType::Help),
        KeyCode::Char('u') => app.toggle_temp_unit(),
        KeyCode::Char('r') => {
            if app.page == Page::Processes {
                app.process_sort_reverse = !app.process_sort_reverse;
            } else if app.page == Page::Hardware || app.page == Page::Device {
                app.request_rescan = true;
                collector.rescan_hardware();
                *snapshot = collector.sample(&app.page.needs());
                app.add_toast("Rescanned hardware & ports", ToastKind::Info);
            }
        }

        // Processes page specific keys
        KeyCode::Char('/') if app.page == Page::Processes => {
            app.filter_active = true;
        }
        KeyCode::Char('s') if app.page == Page::Processes => {
            app.process_sort = app.process_sort.cycle();
            app.add_toast(
                format!("Sorted by {}", app.process_sort.label()),
                ToastKind::Info,
            );
        }
        KeyCode::Char('K') | KeyCode::Char('k') if shift && app.page == Page::Processes => {
            if let Some(p) = snapshot.processes.get(app.process_selected) {
                app.active_modal = Some(ModalType::ConfirmKill {
                    pid: p.pid,
                    name: p.name.clone(),
                    escalate_sigkill: true,
                });
            }
        }
        KeyCode::Char('k') if app.page == Page::Processes => {
            // Check if Shift was pressed or regular 'k' for navigation
            if let Some(p) = snapshot.processes.get(app.process_selected) {
                app.active_modal = Some(ModalType::ConfirmKill {
                    pid: p.pid,
                    name: p.name.clone(),
                    escalate_sigkill: false,
                });
            }
        }

        // Navigation (Up / Down / j / k / PgUp / PgDn / g / G)
        KeyCode::Up | KeyCode::Char('k') => {
            if app.page == Page::Processes {
                app.process_selected = app.process_selected.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.page == Page::Processes {
                app.process_selected =
                    (app.process_selected + 1).min(snapshot.processes.len().saturating_sub(1));
            }
        }
        KeyCode::PageUp => {
            if app.page == Page::Processes {
                app.process_selected = app.process_selected.saturating_sub(15);
            }
        }
        KeyCode::PageDown => {
            if app.page == Page::Processes {
                app.process_selected =
                    (app.process_selected + 15).min(snapshot.processes.len().saturating_sub(1));
            }
        }
        KeyCode::Char('g') => {
            if app.page == Page::Processes {
                app.process_selected = 0;
            }
        }
        KeyCode::Char('G') => {
            if app.page == Page::Processes {
                app.process_selected = snapshot.processes.len().saturating_sub(1);
            }
        }

        // Network page interface cycle
        KeyCode::Char('n') if app.page == Page::Network => {
            if !snapshot.network.interfaces.is_empty() {
                app.net_iface_index = (app.net_iface_index + 1) % snapshot.network.interfaces.len();
            }
        }

        // Disks page volume switch & eject
        KeyCode::Char('h') if app.page == Page::Disks => {
            app.disk_selected = app.disk_selected.saturating_sub(1);
        }
        KeyCode::Char('l') if app.page == Page::Disks => {
            if !snapshot.disks.volumes.is_empty() {
                app.disk_selected = (app.disk_selected + 1).min(snapshot.disks.volumes.len() - 1);
            }
        }
        KeyCode::Char('e') if app.page == Page::Disks => {
            if let Some(v) = snapshot.disks.volumes.get(app.disk_selected) {
                app.active_modal = Some(ModalType::ConfirmEject {
                    mount: v.mount.clone(),
                    device: v.device.clone(),
                });
            }
        }

        // Device page port selection (1..7)
        KeyCode::Char(c) if app.page == Page::Device && c.is_ascii_digit() => {
            if let Some(digit) = c.to_digit(10)
                && (1..=7).contains(&digit)
            {
                app.selected_port = Some(digit as u8);
            }
        }
        KeyCode::Esc if app.page == Page::Device => {
            app.selected_port = None;
        }

        _ => {}
    }
}

fn handle_mouse_event(app: &mut App, kind: MouseEventKind, col: u16, row: u16) {
    match kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Click on tab bar (row 0)
            if row == 0 {
                if col < 14 {
                    app.page = Page::Overview;
                } else if col < 28 {
                    app.page = Page::Processes;
                } else if col < 40 {
                    app.page = Page::Network;
                } else if col < 50 {
                    app.page = Page::Disks;
                } else if col < 63 {
                    app.page = Page::Hardware;
                } else if col < 74 {
                    app.page = Page::Device;
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if app.page == Page::Processes {
                app.process_selected = app.process_selected.saturating_add(3);
            }
        }
        MouseEventKind::ScrollUp => {
            if app.page == Page::Processes {
                app.process_selected = app.process_selected.saturating_sub(3);
            }
        }
        _ => {}
    }
}

/// Safely sends SIGTERM or SIGKILL to process using `rustix`.
fn dispatch_kill(app: &mut App, pid: u32, name: &str, use_sigkill: bool) {
    #[cfg(unix)]
    {
        use rustix::process::{Pid, Signal, kill_process};
        let raw_pid = pid as i32;
        if let Some(target_pid) = Pid::from_raw(raw_pid) {
            let sig = if use_sigkill {
                Signal::KILL
            } else {
                Signal::TERM
            };
            match kill_process(target_pid, sig) {
                Ok(()) => {
                    let s = if use_sigkill { "SIGKILL" } else { "SIGTERM" };
                    app.add_toast(format!("Sent {s} to {name} ({pid})"), ToastKind::Success);
                }
                Err(e) => {
                    app.add_toast(format!("Failed to kill {name}: {e}"), ToastKind::Error);
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        app.add_toast(
            "Process termination not supported on this OS",
            ToastKind::Warning,
        );
    }
}

/// Safely unmounts volume.
fn dispatch_eject(app: &mut App, mount: &str, _device: &str) {
    #[cfg(target_os = "macos")]
    {
        match std::process::Command::new("diskutil")
            .args(["unmount", mount])
            .output()
        {
            Ok(output) if output.status.success() => {
                app.add_toast(format!("Unmounted {mount}"), ToastKind::Success);
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                app.add_toast(format!("Eject failed: {err}"), ToastKind::Error);
            }
            Err(e) => {
                app.add_toast(format!("Eject error: {e}"), ToastKind::Error);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("umount").arg(mount).output() {
            Ok(output) if output.status.success() => {
                app.add_toast(format!("Unmounted {mount}"), ToastKind::Success);
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                app.add_toast(format!("Eject failed: {err}"), ToastKind::Error);
            }
            Err(e) => {
                app.add_toast(format!("Eject error: {e}"), ToastKind::Error);
            }
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        app.add_toast("Eject not supported on this OS", ToastKind::Warning);
    }
}
