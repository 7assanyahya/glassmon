//! Advanced Disks page with storage breakdown, I/O throughput, SMART health and volumes table.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, Widget};

use crate::app::App;
use crate::model::{HealthStatus, Snapshot, StorageCategory};
use crate::ui::widgets::bar::{Segment, StackedBar};
use crate::ui::widgets::{Card, Pill, SmoothBar, Sparkline};
use crate::units;

pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(46), // Top: Storage breakdown & Throughput
            Constraint::Percentage(54), // Bottom: Volumes table, SMART & Top disk activity
        ])
        .split(area);

    render_top_row(app, snapshot, chunks[0], buf);
    render_bottom_row(app, snapshot, chunks[1], buf);
}

fn render_top_row(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Storage breakdown
            Constraint::Percentage(45), // Throughput & I/O tiles
        ])
        .split(area);

    render_storage_breakdown(app, snapshot, chunks[0], buf);
    render_throughput_card(app, snapshot, chunks[1], buf);
}

fn render_storage_breakdown(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Storage Breakdown", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    let volumes = &snapshot.disks.volumes;
    let sel_vol = if volumes.is_empty() {
        None
    } else {
        Some(&volumes[app.disk_selected % volumes.len()])
    };

    let mount_str = sel_vol.map(|v| v.mount.as_str()).unwrap_or("/");
    let breakdown = snapshot.disks.breakdown_for(mount_str);

    let mut y = inner.y;

    // Line 1: Drive name & switcher hint
    let title_line = Line::from(vec![
        Span::styled(
            format!("Drive: {} ", sel_vol.map(|v| v.name.as_str()).unwrap_or("Internal SSD")),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("({}) [h/l switch]", mount_str),
            Style::default().fg(theme.secondary),
        ),
    ]);
    buf.set_line(inner.x, y, &title_line, inner.width);
    y += 1;

    // Line 2: Big used figure
    if let Some(vol) = sel_vol {
        let used_str = units::bytes(vol.used());
        let total_str = units::bytes(vol.total);
        let pct_str = format!("{:.1}%", vol.used_pct());

        let used_line = Line::from(vec![
            Span::styled("Used: ", Style::default().fg(theme.secondary)),
            Span::styled(format!("{used_str} "), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(format!("of {total_str} ({pct_str})"), Style::default().fg(theme.secondary)),
        ]);
        buf.set_line(inner.x, y, &used_line, inner.width);
        y += 1;
    }

    // Line 3: Stacked Bar
    let bar_rect = Rect {
        x: inner.x,
        y,
        width: inner.width,
        height: 1,
    };
    y += 2;

    let cat_colors = |cat: StorageCategory| -> Color {
        match cat {
            StorageCategory::Apps => theme.blue,
            StorageCategory::Developer => theme.purple,
            StorageCategory::Media => theme.pink,
            StorageCategory::Documents => theme.orange,
            StorageCategory::System => theme.secondary,
            StorageCategory::Other => theme.cyan,
            StorageCategory::Used => theme.blue,
            StorageCategory::Free => theme.empty_track,
        }
    };

    if let Some(bd) = breakdown && !bd.segments.is_empty() {
        let segs: Vec<Segment> = bd
            .segments
            .iter()
            .map(|(cat, val)| Segment {
                label: cat.label(),
                value: *val,
                color: cat_colors(*cat),
            })
            .collect();
        StackedBar::new(segs, theme.empty_track).render(bar_rect, buf);

        // 4-column legend
        let legend_items: Vec<_> = bd.segments.iter().filter(|(c, _)| *c != StorageCategory::Free).collect();
        let cols = 4;
        let col_w = inner.width / cols;

        for (i, (cat, bytes_val)) in legend_items.iter().enumerate() {
            let col = i as u16 % cols;
            let row = i as u16 / cols;
            let item_x = inner.x + col * col_w;
            let item_y = y + row;

            if item_y < inner.y + inner.height {
                let color = cat_colors(*cat);
                let label = format!("{}: {}", cat.label(), units::bytes(*bytes_val));
                let line = Line::from(vec![
                    Span::styled("■ ", Style::default().fg(color)),
                    Span::styled(label, Style::default().fg(theme.secondary)),
                ]);
                buf.set_line(item_x, item_y, &line, col_w);
            }
        }
    } else if let Some(vol) = sel_vol {
        // Fallback used / free bar
        let col = theme.usage_color(vol.used_pct());
        SmoothBar::new(vol.used_pct() / 100.0, col, theme.empty_track).render(bar_rect, buf);
    }
}

fn render_throughput_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Throughput & Latency", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Rates text
            Constraint::Length(inner.height.saturating_sub(5).min(3)), // Sparkline
            Constraint::Min(2), // Metrics tiles
        ])
        .split(inner);

    let io = &snapshot.disks.io;
    let r_str = units::rate(io.read_rate);
    let w_str = units::rate(io.write_rate);

    let text_line = Line::from(vec![
        Span::styled("Read: ", Style::default().fg(theme.secondary)),
        Span::styled(format!("{r_str:<10}  "), Style::default().fg(theme.green).add_modifier(Modifier::BOLD)),
        Span::styled("Write: ", Style::default().fg(theme.secondary)),
        Span::styled(w_str, Style::default().fg(theme.orange).add_modifier(Modifier::BOLD)),
    ]);
    buf.set_line(chunks[0].x, chunks[0].y, &text_line, chunks[0].width);

    // Sparkline for Read
    if chunks[1].height > 0 {
        let r_u64: Vec<u64> = io.read_history.iter().map(|&v| (v / 1024.0) as u64).collect();
        Sparkline::new(&r_u64, theme.green).render(chunks[1], buf);
    }

    // Tiles: IOPS, Latency, Queue depth, Busy%
    if chunks[2].height >= 2 {
        let iops_str = io.iops.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "n/a".into());
        let lat_str = io.latency_ms.map(|v| format!("{:.2} ms", v)).unwrap_or_else(|| "n/a".into());
        let q_str = io.queue_depth.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "n/a".into());
        let busy_str = io.busy_pct.map(|v| format!("{:.1}%", v)).unwrap_or_else(|| "n/a".into());

        let tile_line_1 = Line::from(vec![
            Span::styled("IOPS: ", Style::default().fg(theme.secondary)),
            Span::styled(format!("{iops_str:<8}  "), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("Latency: ", Style::default().fg(theme.secondary)),
            Span::styled(lat_str, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]);
        let tile_line_2 = Line::from(vec![
            Span::styled("Queue: ", Style::default().fg(theme.secondary)),
            Span::styled(format!("{q_str:<7}  "), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("Busy: ", Style::default().fg(theme.secondary)),
            Span::styled(busy_str, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]);

        buf.set_line(chunks[2].x, chunks[2].y, &tile_line_1, chunks[2].width);
        buf.set_line(chunks[2].x, chunks[2].y + 1, &tile_line_2, chunks[2].width);
    }
}

fn render_bottom_row(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45), // Volumes table
            Constraint::Percentage(30), // SMART card
            Constraint::Percentage(25), // Top disk activity
        ])
        .split(area);

    render_volumes_table(app, snapshot, chunks[0], buf);
    render_smart_card(app, snapshot, chunks[1], buf);
    render_disk_activity_table(app, snapshot, chunks[2], buf);
}

fn render_volumes_table(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Physical Volumes", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let header_cells = ["MOUNT", "FS", "SIZE", "USED", "FREE", "USE%"]
        .into_iter()
        .map(|h| Cell::from(Span::styled(h, Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
    let header = Row::new(header_cells).height(1);

    let volumes = &snapshot.disks.volumes;
    let selected_idx = if volumes.is_empty() { 0 } else { app.disk_selected % volumes.len() };

    let rows: Vec<Row> = volumes
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let is_selected = idx == selected_idx;
            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default().fg(theme.text)
            };

            let pct = v.used_pct();
            let col = if is_selected { Color::White } else { theme.usage_color(pct) };

            Row::new(vec![
                Cell::from(Span::styled(units::truncate(&v.mount, 12), Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(v.fs.clone()),
                Cell::from(units::bytes(v.total)),
                Cell::from(units::bytes(v.used())),
                Cell::from(units::bytes(v.available)),
                Cell::from(Span::styled(format!("{pct:.0}%"), Style::default().fg(col))),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(6),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Min(6),
    ];

    let table = Table::new(rows, widths).header(header);
    Widget::render(table, inner, buf);
}

fn render_smart_card(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("SMART Health", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let smart = snapshot.disks.smart.first();
    let mut y = inner.y;

    if let Some(s) = smart {
        // Status pill
        let (pill_text, pill_fg, pill_bg) = match s.status {
            HealthStatus::Verified => (" Verified ", Color::White, theme.green),
            HealthStatus::Warning => (" Warning ", Color::Black, theme.orange),
            HealthStatus::Unknown => (" Unknown ", theme.text, theme.empty_track),
        };

        let pill_spans = Pill::new(pill_text, pill_fg, pill_bg);
        let mut status_line = vec![
            Span::styled("Status: ", Style::default().fg(theme.secondary)),
        ];
        status_line.extend(pill_spans);
        buf.set_line(inner.x, y, &Line::from(status_line), inner.width);
        y += 1;

        let entries = [
            ("Temp", s.temperature_c.map(|t| units::temp(t, app.fahrenheit)).unwrap_or_else(|| "n/a".into())),
            ("Power Hours", s.power_on_hours.map(|h| format!("{h}h")).unwrap_or_else(|| "n/a".into())),
            ("Data Written", s.data_written.map(units::bytes).unwrap_or_else(|| "n/a".into())),
            ("Wear Level", s.wear_pct.map(|p| format!("{p:.0}%")).unwrap_or_else(|| "n/a".into())),
            ("Spare Available", s.available_spare_pct.map(|p| format!("{p:.0}%")).unwrap_or_else(|| "n/a".into())),
            ("Media Errors", s.media_errors.map(|e| e.to_string()).unwrap_or_else(|| "n/a".into())),
        ];

        for (k, v) in entries {
            if y >= inner.y + inner.height {
                break;
            }
            let line = Line::from(vec![
                Span::styled(format!("{:<15}", k), Style::default().fg(theme.secondary)),
                Span::styled(v, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            ]);
            buf.set_line(inner.x, y, &line, inner.width);
            y += 1;
        }
    } else {
        let msg = Line::from(Span::styled("No SMART device found", Style::default().fg(theme.secondary)));
        buf.set_line(inner.x, y, &msg, inner.width);
    }
}

fn render_disk_activity_table(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Top Disk I/O", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let mut procs = snapshot.processes.clone();
    procs.sort_by(|a, b| {
        let a_tot = a.disk_read_rate + a.disk_write_rate;
        let b_tot = b.disk_read_rate + b.disk_write_rate;
        b_tot.partial_cmp(&a_tot).unwrap_or(std::cmp::Ordering::Equal)
    });

    let header_cells = ["PROCESS", "READ/s", "WRITE/s"]
        .into_iter()
        .map(|h| Cell::from(Span::styled(h, Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = procs
        .iter()
        .take(inner.height.saturating_sub(1) as usize)
        .map(|p| {
            Row::new(vec![
                Cell::from(units::truncate(&p.name, 10)),
                Cell::from(Span::styled(units::rate(p.disk_read_rate), Style::default().fg(theme.green))),
                Cell::from(Span::styled(units::rate(p.disk_write_rate), Style::default().fg(theme.orange))),
            ])
            .style(Style::default().fg(theme.text))
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths).header(header);
    Widget::render(table, inner, buf);
}
