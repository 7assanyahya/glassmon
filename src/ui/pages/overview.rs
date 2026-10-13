//! Overview page rendering.
//!
//! Target 120x36:
//! - Left (~36%): CPU card (sparkline + 2-col core grid), Memory card (RAM/Swap/Pressure), Network card.
//! - Right: Processes table filling top height, then Disks & Sensors side by side.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, Widget};

use crate::app::App;
use crate::model::Snapshot;
use crate::ui::widgets::{Card, SmoothBar, Sparkline};
use crate::units;

pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    // Split left (~36%) and right (~64%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(area);

    render_left_col(app, snapshot, chunks[0], buf);
    render_right_col(app, snapshot, chunks[1], buf);
}

fn render_left_col(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(42), // CPU
            Constraint::Percentage(34), // Memory
            Constraint::Percentage(24), // Network
        ])
        .split(area);

    render_cpu_card(app, snapshot, chunks[0], buf);
    render_mem_card(app, snapshot, chunks[1], buf);
    render_net_card(app, snapshot, chunks[2], buf);
}

fn render_cpu_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("CPU Usage", theme).key_badge("1");
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Overall stats
            Constraint::Length(inner.height.saturating_sub(6).min(4)), // Sparkline
            Constraint::Min(2),    // Cores grid
        ])
        .split(inner);

    // Header line
    let cpu_pct = snapshot.cpu.usage as f64;
    let cpu_color = theme.usage_color(cpu_pct);
    let brand = snapshot.cpu.brand.as_deref().unwrap_or("Processor");
    let header_line = Line::from(vec![
        Span::styled(
            format!("{brand} "),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{cpu_pct:.1}%"),
            Style::default().fg(cpu_color).add_modifier(Modifier::BOLD),
        ),
    ]);
    buf.set_line(chunks[0].x, chunks[0].y, &header_line, chunks[0].width);

    // Sparkline
    if chunks[1].height > 0 {
        let u64_data: Vec<u64> = snapshot.cpu.history.iter().map(|&v| v as u64).collect();
        Sparkline::new(&u64_data, cpu_color).render(chunks[1], buf);
    }

    // Cores grid: 2 columns
    let cores = &snapshot.cpu.cores;
    if !cores.is_empty() && chunks[2].height > 0 {
        let half_w = chunks[2].width / 2;
        let visible_rows = chunks[2].height as usize;

        for (idx, core) in cores.iter().take(visible_rows * 2).enumerate() {
            let col = idx % 2;
            let row = idx / 2;
            let row_y = chunks[2].y + row as u16;
            let col_x = chunks[2].x + (col as u16 * half_w);

            let core_area = Rect {
                x: col_x,
                y: row_y,
                width: half_w.saturating_sub(1),
                height: 1,
            };

            let label = format!("{:>3} ", core.label);
            let bar_w = core_area.width.saturating_sub(10);

            // Print core label
            buf.set_string(
                core_area.x,
                core_area.y,
                &label,
                Style::default().fg(theme.secondary),
            );

            // Print bar
            if bar_w > 2 {
                let bar_rect = Rect {
                    x: core_area.x + 4,
                    y: core_area.y,
                    width: bar_w,
                    height: 1,
                };
                let c_color = theme.usage_color(core.usage as f64);
                SmoothBar::new(core.usage as f64 / 100.0, c_color, theme.empty_track)
                    .render(bar_rect, buf);
            }

            // Print percentage
            let pct_str = format!("{:>3.0}%", core.usage);
            buf.set_string(
                core_area.x + core_area.width.saturating_sub(4),
                core_area.y,
                &pct_str,
                Style::default().fg(theme.text),
            );
        }
    }
}

fn render_mem_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Memory & Swap", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    let mem = &snapshot.memory;
    let ram_pct = mem.used_pct();
    let swap_pct = mem.swap_pct();
    let ram_col = theme.usage_color(ram_pct);
    let swap_col = theme.usage_color(swap_pct);

    let mut lines = Vec::new();

    // RAM line
    lines.push(Line::from(vec![
        Span::styled("RAM   ", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}/{} ({:.1}%)", units::bytes_bin(mem.used), units::bytes_bin(mem.total), ram_pct),
            Style::default().fg(theme.secondary),
        ),
    ]));

    // Swap line
    lines.push(Line::from(vec![
        Span::styled("Swap  ", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}/{} ({:.1}%)", units::bytes_bin(mem.swap_used), units::bytes_bin(mem.swap_total), swap_pct),
            Style::default().fg(swap_col),
        ),
    ]));

    if let Some(press) = mem.pressure {
        let press_col = theme.usage_color(press as f64);
        lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{press:.0}%"),
                Style::default().fg(press_col),
            ),
        ]));
    }

    for (i, line) in lines.iter().enumerate() {
        if (i as u16) < inner.height {
            buf.set_line(inner.x, inner.y + i as u16, line, inner.width);
        }
    }

    // Render bars below
    let bar_y = inner.y + lines.len() as u16;
    if bar_y < inner.y + inner.height {
        let bar_rect = Rect {
            x: inner.x,
            y: bar_y,
            width: inner.width,
            height: 1,
        };
        SmoothBar::new(ram_pct / 100.0, ram_col, theme.empty_track).render(bar_rect, buf);
    }
}

fn render_net_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Network Throughput", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let net = &snapshot.network;
    let rx_str = units::rate(net.rx_rate);
    let tx_str = units::rate(net.tx_rate);

    let text_line = Line::from(vec![
        Span::styled("▼ ", Style::default().fg(theme.green)),
        Span::styled(format!("{rx_str:<10}  "), Style::default().fg(theme.text)),
        Span::styled("▲ ", Style::default().fg(theme.orange)),
        Span::styled(tx_str, Style::default().fg(theme.text)),
    ]);
    buf.set_line(inner.x, inner.y, &text_line, inner.width);

    // Sparkline for RX rate
    if inner.height > 2 {
        let spark_rect = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };
        let u64_rx: Vec<u64> = net.rx_history.iter().map(|&v| (v / 1024.0) as u64).collect();
        Sparkline::new(&u64_rx, theme.green).render(spark_rect, buf);
    }
}

fn render_right_col(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(62), // Processes table
            Constraint::Percentage(38), // Disks & Sensors side by side
        ])
        .split(area);

    render_proc_table(app, snapshot, chunks[0], buf);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_disk_mini(app, snapshot, bottom_chunks[0], buf);
    render_sensor_mini(app, snapshot, bottom_chunks[1], buf);
}

fn render_proc_table(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Top Processes", theme).key_badge("2");
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let header_cells = ["PID", "NAME", "USER", "CPU%", "MEM"]
        .into_iter()
        .map(|h| Cell::from(Span::styled(h, Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = snapshot
        .processes
        .iter()
        .take(inner.height.saturating_sub(1) as usize)
        .map(|p| {
            let cpu_col = theme.usage_color(p.cpu as f64);
            let user_str = p.user.as_deref().unwrap_or("-");
            Row::new(vec![
                Cell::from(format!("{}", p.pid)),
                Cell::from(Span::styled(units::truncate(&p.name, 18), Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(units::truncate(user_str, 12)),
                Cell::from(Span::styled(format!("{:.1}%", p.cpu), Style::default().fg(cpu_col))),
                Cell::from(units::bytes(p.memory)),
            ])
            .style(Style::default().fg(theme.text))
        })
        .collect();

    let widths = [
        Constraint::Length(7),
        Constraint::Min(16),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths).header(header);
    Widget::render(table, inner, buf);
}

fn render_disk_mini(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Disks", theme).key_badge("4");
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut y = inner.y;
    for vol in snapshot.disks.volumes.iter().take(inner.height as usize / 2) {
        if y >= inner.y + inner.height {
            break;
        }
        let line = Line::from(vec![
            Span::styled(format!("{:<14} ", units::truncate(&vol.name, 14)), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} / {}", units::bytes(vol.used()), units::bytes(vol.total)), Style::default().fg(theme.secondary)),
        ]);
        buf.set_line(inner.x, y, &line, inner.width);
        y += 1;

        if y < inner.y + inner.height {
            let bar_rect = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            let pct = vol.used_pct();
            let col = theme.usage_color(pct);
            SmoothBar::new(pct / 100.0, col, theme.empty_track).render(bar_rect, buf);
            y += 1;
        }
    }
}

fn render_sensor_mini(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Sensors & Fans", theme).key_badge("5");
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut y = inner.y;
    for sensor in snapshot.sensors.temps.iter().take(inner.height as usize) {
        if y >= inner.y + inner.height {
            break;
        }
        let col = theme.temp_color(sensor.celsius);
        let temp_str = units::temp(sensor.celsius, app.fahrenheit);
        let line = Line::from(vec![
            Span::styled(format!("{:<16} ", units::truncate(&sensor.label, 16)), Style::default().fg(theme.text)),
            Span::styled(temp_str, Style::default().fg(col).add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(inner.x, y, &line, inner.width);
        y += 1;
    }
}
