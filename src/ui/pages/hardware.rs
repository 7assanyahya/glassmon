//! Hardware & Sensors page with system specifications, thermals, fans, battery, Wi-Fi/Bluetooth and storage.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::App;
use crate::model::{HealthStatus, PowerSource, Snapshot};
use crate::ui::widgets::{Card, SmoothBar};
use crate::units;

pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left: System & Battery
            Constraint::Percentage(38), // Middle: Thermals, Fans & Connectivity
            Constraint::Percentage(32), // Right: Ports & Storage
        ])
        .split(area);

    render_left_col(app, snapshot, chunks[0], buf);
    render_middle_col(app, snapshot, chunks[1], buf);
    render_right_col(app, snapshot, chunks[2], buf);
}

fn render_left_col(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // System card
            Constraint::Percentage(50), // Battery & power card
        ])
        .split(area);

    render_system_card(app, snapshot, chunks[0], buf);
    render_battery_card(app, snapshot, chunks[1], buf);
}

fn render_system_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("System Architecture", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let hw = &snapshot.hardware;
    let mut y = inner.y;

    let entries = [
        ("Model", hw.model_name.as_deref().unwrap_or("MacBook Pro")),
        ("Identifier", hw.model_id.as_deref().unwrap_or("Unknown")),
        ("Processor", hw.chip.as_deref().or(snapshot.cpu.brand.as_deref()).unwrap_or("Apple Silicon")),
        ("CPU Cores", &hw.cores.map(|c| format!("{} ({}P + {}E)", c.total, c.performance.unwrap_or(0), c.efficiency.unwrap_or(0))).unwrap_or_else(|| format!("{}", snapshot.cpu.cores.len()))),
        ("GPU Cores", &hw.gpu_cores.map(|g| format!("{g} Cores")).unwrap_or_else(|| "Integrated".into())),
        ("Memory", &units::bytes_bin(hw.memory_total.unwrap_or(snapshot.memory.total))),
        ("Display", hw.display.as_deref().unwrap_or("Built-in Display")),
        ("Uptime", &snapshot.host.uptime_secs.map(units::duration).unwrap_or_else(|| "-".into())),
    ];

    for (k, v) in entries {
        if y >= inner.y + inner.height {
            break;
        }
        let line = Line::from(vec![
            Span::styled(format!("{:<11}", k), Style::default().fg(theme.secondary)),
            Span::styled(units::truncate(v, inner.width.saturating_sub(12) as usize), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(inner.x, y, &line, inner.width);
        y += 1;
    }
}

fn render_battery_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Battery & Power", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    if let Some(bat) = &snapshot.battery {
        let pct_str = format!("{:.0}%", bat.percent);
        let col = theme.usage_color((100.0 - bat.percent) as f64);

        // Line 1: State & Percent
        let status_line = Line::from(vec![
            Span::styled("Charge: ", Style::default().fg(theme.secondary)),
            Span::styled(format!("{pct_str}  "), Style::default().fg(col).add_modifier(Modifier::BOLD)),
            Span::styled(bat.state.label(), Style::default().fg(theme.cyan)),
        ]);
        buf.set_line(inner.x, y, &status_line, inner.width);
        y += 1;

        // Line 2: Bar
        if y < inner.y + inner.height {
            let bar_rect = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            SmoothBar::new(bat.percent as f64 / 100.0, col, theme.empty_track).render(bar_rect, buf);
            y += 2;
        }

        let power_str = bat.power_w.map(|w| format!("{:.1} W", w)).unwrap_or_else(|| "n/a".into());
        let time_str = if let Some(t) = bat.time_to_empty_secs {
            format!("{} remaining", units::duration(t))
        } else if let Some(t) = bat.time_to_full_secs {
            format!("{} to full", units::duration(t))
        } else {
            "n/a".into()
        };

        let entries = [
            ("Power Source", match bat.source {
                Some(PowerSource::Adapter) => "Power Adapter",
                Some(PowerSource::Battery) => "Battery",
                None => "n/a",
            }),
            ("Draw", &power_str),
            ("Time", &time_str),
            ("Cycles", &bat.cycle_count.map(|c| c.to_string()).unwrap_or_else(|| "n/a".into())),
            ("Health", &bat.health_pct.map(|h| format!("{h:.0}%")).unwrap_or_else(|| "n/a".into())),
        ];

        for (k, v) in entries {
            if y >= inner.y + inner.height {
                break;
            }
            let line = Line::from(vec![
                Span::styled(format!("{:<13}", k), Style::default().fg(theme.secondary)),
                Span::styled(v, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            ]);
            buf.set_line(inner.x, y, &line, inner.width);
            y += 1;
        }
    } else {
        let msg = Line::from(Span::styled("No battery detected (Desktop / AC powered)", Style::default().fg(theme.secondary)));
        buf.set_line(inner.x, y, &msg, inner.width);
    }
}

fn render_middle_col(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // Temperatures
            Constraint::Percentage(25), // Fans
            Constraint::Percentage(30), // Wi-Fi & Bluetooth
        ])
        .split(area);

    render_thermals_card(app, snapshot, chunks[0], buf);
    render_fans_card(app, snapshot, chunks[1], buf);
    render_wireless_card(app, snapshot, chunks[2], buf);
}

fn render_thermals_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Temperatures [u switch]", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    for sensor in snapshot.sensors.temps.iter().take(inner.height as usize) {
        if y >= inner.y + inner.height {
            break;
        }
        let col = theme.temp_color(sensor.celsius);
        let temp_str = units::temp(sensor.celsius, app.fahrenheit);

        let lbl_w = 16.min(inner.width.saturating_sub(12) as usize);
        let bar_w = inner.width.saturating_sub(lbl_w as u16 + 8);

        // Print label
        buf.set_string(
            inner.x,
            y,
            &format!("{:<width$}", units::truncate(&sensor.label, lbl_w), width = lbl_w),
            Style::default().fg(theme.text),
        );

        // Print bar
        if bar_w > 4 {
            let bar_rect = Rect {
                x: inner.x + lbl_w as u16 + 1,
                y,
                width: bar_w,
                height: 1,
            };
            let ratio = (sensor.celsius as f64 / 100.0).clamp(0.0, 1.0);
            SmoothBar::new(ratio, col, theme.empty_track).render(bar_rect, buf);
        }

        // Print temperature value
        buf.set_string(
            inner.x + inner.width.saturating_sub(6),
            y,
            &format!("{:>5}", temp_str),
            Style::default().fg(col).add_modifier(Modifier::BOLD),
        );

        y += 1;
    }
}

fn render_fans_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Cooling Fans", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 1 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    let fans = &snapshot.sensors.fans;

    if fans.is_empty() {
        let msg = Line::from(Span::styled("No active fans detected (Passively cooled)", Style::default().fg(theme.secondary)));
        buf.set_line(inner.x, y, &msg, inner.width);
        return;
    }

    for fan in fans.iter().take(inner.height as usize) {
        if y >= inner.y + inner.height {
            break;
        }

        let max = fan.max_rpm.unwrap_or(6000);
        let ratio = (fan.rpm as f64 / max as f64).clamp(0.0, 1.0);
        let rpm_str = format!("{} RPM", fan.rpm);

        let lbl_w = 14.min(inner.width.saturating_sub(14) as usize);
        let bar_w = inner.width.saturating_sub(lbl_w as u16 + 12);

        buf.set_string(
            inner.x,
            y,
            &format!("{:<width$}", units::truncate(&fan.label, lbl_w), width = lbl_w),
            Style::default().fg(theme.text),
        );

        if bar_w > 4 {
            let bar_rect = Rect {
                x: inner.x + lbl_w as u16 + 1,
                y,
                width: bar_w,
                height: 1,
            };
            SmoothBar::new(ratio, theme.cyan, theme.empty_track).render(bar_rect, buf);
        }

        buf.set_string(
            inner.x + inner.width.saturating_sub(10),
            y,
            &format!("{:>9}", rpm_str),
            Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD),
        );

        y += 1;
    }
}

fn render_wireless_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Wi-Fi & Bluetooth", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    if let Some(wifi) = &snapshot.hardware.wifi {
        let ssid = wifi.ssid.as_deref().unwrap_or("Connected");
        let signal = wifi.signal_dbm.map(|s| format!("{s} dBm")).unwrap_or_else(|| "n/a".into());
        let speed = wifi.link_mbps.map(|m| format!("{m:.0} Mbps")).unwrap_or_else(|| "n/a".into());

        let line1 = Line::from(vec![
            Span::styled("Wi-Fi: ", Style::default().fg(theme.secondary)),
            Span::styled(format!("{ssid}  "), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(format!("({signal}, {speed})"), Style::default().fg(theme.green)),
        ]);
        buf.set_line(inner.x, y, &line1, inner.width);
        y += 1;
    }

    // Bluetooth
    let bt_devices: Vec<_> = snapshot.hardware.bluetooth.iter().filter(|d| d.connected).collect();
    let bt_count = bt_devices.len();
    let bt_summary = if bt_count == 0 {
        "No devices connected".to_string()
    } else {
        bt_devices.iter().map(|d| d.name.as_str()).collect::<Vec<_>>().join(", ")
    };

    let line2 = Line::from(vec![
        Span::styled("BT:    ", Style::default().fg(theme.secondary)),
        Span::styled(units::truncate(&bt_summary, inner.width.saturating_sub(8) as usize), Style::default().fg(theme.text)),
    ]);
    buf.set_line(inner.x, y, &line2, inner.width);
}

fn render_right_col(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Ports card
            Constraint::Percentage(45), // Storage devices card
        ])
        .split(area);

    render_ports_card(app, snapshot, chunks[0], buf);
    render_storage_card(app, snapshot, chunks[1], buf);
}

fn render_ports_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let ports = &snapshot.ports;
    let in_use = ports.in_use();
    let total = ports.ports.len();

    let title = format!("Ports ({} of {} in use) [r rescan]", in_use, total);
    let card = Card::new(&title, theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    for port in ports.ports.iter().take(inner.height as usize) {
        if y >= inner.y + inner.height {
            break;
        }

        let is_connected = port.connected == Some(true);
        let dot_col = if is_connected { theme.green } else { theme.secondary };
        let dev_text = port.device.as_deref().or(port.speed.as_deref()).unwrap_or("Empty");

        let line = Line::from(vec![
            Span::styled("● ", Style::default().fg(dot_col)),
            Span::styled(format!("{:<14} ", units::truncate(&port.name, 14)), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(units::truncate(dev_text, inner.width.saturating_sub(20) as usize), Style::default().fg(if is_connected { theme.cyan } else { theme.secondary })),
        ]);
        buf.set_line(inner.x, y, &line, inner.width);
        y += 1;
    }
}

fn render_storage_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Internal & External Storage", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    for dev in snapshot.hardware.storage.iter().take(inner.height as usize / 2) {
        if y >= inner.y + inner.height {
            break;
        }

        let health_col = match dev.health {
            HealthStatus::Verified => theme.green,
            HealthStatus::Warning => theme.orange,
            HealthStatus::Unknown => theme.secondary,
        };

        let line1 = Line::from(vec![
            Span::styled(format!("{:<18} ", units::truncate(&dev.name, 18)), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("● Verified", Style::default().fg(health_col)),
        ]);
        buf.set_line(inner.x, y, &line1, inner.width);
        y += 1;

        if y < inner.y + inner.height {
            let iface_str = dev.interface.as_deref().unwrap_or("Internal NVMe");
            let size_str = dev.size.map(units::bytes).unwrap_or_else(|| "-".into());
            let line2 = Line::from(vec![
                Span::styled(format!("  {} ({})", iface_str, size_str), Style::default().fg(theme.secondary)),
            ]);
            buf.set_line(inner.x, y, &line2, inner.width);
            y += 1;
        }
    }
}
