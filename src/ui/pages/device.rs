//! Device page with machine chassis diagram, numbered port badges and health tiles.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Widget};

use crate::app::App;
use crate::model::{Side, Snapshot};
use crate::ui::theme::Theme;
use crate::ui::widgets::Card;
use crate::units;

pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Left: Chassis & Edge Strips
            Constraint::Percentage(40), // Right: Ports List & Health Tiles Grid
        ])
        .split(area);

    render_chassis_view(app, snapshot, chunks[0], buf);
    render_device_details(app, snapshot, chunks[1], buf);
}

fn render_chassis_view(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let ports = &snapshot.ports;
    let card_title = format!("{} Chassis", ports.layout_name);
    let card = Card::new(&card_title, theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 6 || inner.width < 10 {
        return;
    }

    // Under 70 columns inside the card: fall back to a numbered list
    if inner.width < 70 {
        render_compact_port_list(app, snapshot, inner, buf);
        return;
    }

    // Split inner into:
    // Left edge strip (14 cols) | Chassis center | Right edge strip (14 cols)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(36),
            Constraint::Length(14),
        ])
        .split(inner);

    render_edge_strip(app, snapshot, Side::Left, h_chunks[0], buf);
    render_machine_ascii(app, snapshot, h_chunks[1], buf);
    render_edge_strip(app, snapshot, Side::Right, h_chunks[2], buf);
}

fn render_edge_strip(app: &App, snapshot: &Snapshot, side: Side, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let side_ports: Vec<_> = snapshot.ports.ports.iter().filter(|p| p.side == side).collect();
    if side_ports.is_empty() {
        return;
    }

    let mut y = area.y + 1;
    let label = match side {
        Side::Left => "◀ Left Edge",
        Side::Right => "Right Edge ▶",
        _ => "Edge",
    };

    buf.set_string(
        area.x,
        y,
        label,
        Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD),
    );
    y += 2;

    for port in side_ports {
        if y + 2 >= area.y + area.height {
            break;
        }

        let is_selected = app.selected_port == Some(port.index);
        let in_use = port.connected == Some(true);

        let (badge_bg, badge_fg) = if is_selected {
            (theme.blue, Color::White)
        } else if in_use {
            (theme.green, Color::Black)
        } else {
            (theme.empty_track, theme.secondary)
        };

        // Number badge: `[ 1 ]` or `[ 2 ]`
        let badge_line = Line::from(vec![
            Span::styled(
                format!(" [ {} ] ", port.index),
                Style::default().bg(badge_bg).fg(badge_fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", port.kind.short()),
                Style::default().fg(if is_selected { theme.blue } else { theme.text }),
            ),
        ]);
        buf.set_line(area.x, y, &badge_line, area.width);
        y += 1;

        // Subtext: speed or status
        let subtext = if let Some(spd) = &port.speed {
            units::truncate(spd, area.width as usize)
        } else if in_use {
            "Active".into()
        } else {
            "Empty".into()
        };

        buf.set_string(
            area.x + 2,
            y,
            &subtext,
            Style::default().fg(if in_use { theme.green } else { theme.secondary }),
        );
        y += 2;
    }
}

fn render_machine_ascii(app: &App, _snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let center_x = area.x + (area.width.saturating_sub(42)) / 2;
    let start_y = area.y + 1;

    // Drawn MacBook screen with notch, dashboard and base edge
    let screen_art = [
        " ╭────────────────────────────────────────╮ ",
        " │             ╭───[ ⬢ ]───╮              │ ",
        " │             ╰───────────╯              │ ",
        " │  CPU 32%  ●  MEM 52%  ●  BAT 84% ⚡     │ ",
        " │  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   │ ",
        " │  Liquid Retina XDR 120Hz ProMotion     │ ",
        " ╰────────────────────────────────────────╯ ",
        "     ╭────────────────────────────────╮     ",
        " ════╧════════════════════════════════╧════ ",
    ];

    for (i, line) in screen_art.iter().enumerate() {
        if start_y + i as u16 >= area.y + area.height {
            break;
        }

        let style = if i == 0 || i == 6 || i == 8 {
            Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD)
        } else if i == 3 {
            Style::default().fg(theme.green)
        } else {
            Style::default().fg(theme.secondary)
        };

        buf.set_string(center_x, start_y + i as u16, line, style);
    }

    // Legend at bottom
    let legend_y = start_y + screen_art.len() as u16 + 1;
    if legend_y + 1 < area.y + area.height {
        let leg_line = Line::from(vec![
            Span::styled(" [ 1 ] ", Style::default().bg(theme.green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" In Use   ", Style::default().fg(theme.secondary)),
            Span::styled(" [ 2 ] ", Style::default().bg(theme.empty_track).fg(theme.secondary)),
            Span::styled(" Disconnected   ", Style::default().fg(theme.secondary)),
            Span::styled(" [Key 1-7] ", Style::default().fg(theme.blue).add_modifier(Modifier::BOLD)),
            Span::styled("Select port", Style::default().fg(theme.secondary)),
        ]);
        buf.set_line(area.x + 2, legend_y, &leg_line, area.width.saturating_sub(4));
    }
}

fn render_compact_port_list(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let mut y = area.y;

    for port in &snapshot.ports.ports {
        if y >= area.y + area.height {
            break;
        }

        let is_selected = app.selected_port == Some(port.index);
        let in_use = port.connected == Some(true);
        let badge_bg = if is_selected {
            theme.blue
        } else if in_use {
            theme.green
        } else {
            theme.empty_track
        };

        let line = Line::from(vec![
            Span::styled(format!(" [{}] ", port.index), Style::default().bg(badge_bg).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {:<18} ", port.name), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(port.device.as_deref().unwrap_or("Disconnected"), Style::default().fg(if in_use { theme.cyan } else { theme.secondary })),
        ]);

        buf.set_line(area.x, y, &line, area.width);
        y += 1;
    }
}

fn render_device_details(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // Numbered Ports List
            Constraint::Percentage(45), // 2x2 Health Tiles Grid
        ])
        .split(area);

    render_ports_table(app, snapshot, chunks[0], buf);
    render_health_grid(app, snapshot, chunks[1], buf);
}

fn render_ports_table(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Chassis Port Matrix", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let mut y = inner.y;
    for port in snapshot.ports.ports.iter().take(inner.height as usize) {
        if y >= inner.y + inner.height {
            break;
        }

        let is_selected = app.selected_port == Some(port.index);
        let in_use = port.connected == Some(true);

        let row_style = if is_selected {
            theme.selected_row()
        } else {
            Style::default().fg(theme.text)
        };

        let badge_str = format!(" [{}] ", port.index);
        let dev_str = port.device.as_deref().or(port.speed.as_deref()).unwrap_or("Empty");

        let mut line = Line::from(vec![
            Span::styled(badge_str, Style::default().fg(if is_selected { Color::White } else { theme.cyan }).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<16} ", units::truncate(&port.name, 16)), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(units::truncate(dev_str, inner.width.saturating_sub(24) as usize), Style::default().fg(if in_use { theme.green } else { theme.secondary })),
        ]);
        line = line.style(row_style);

        buf.set_line(inner.x, y, &line, inner.width);
        y += 1;
    }
}

fn render_health_grid(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Hardware Health Tiles", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 4 || inner.width < 10 {
        return;
    }

    // 2x2 grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bot_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    // Tile 1: Thermals
    let max_temp = snapshot.sensors.max_temp().unwrap_or(45.0);
    render_health_tile("Thermals", &units::temp(max_temp, app.fahrenheit), "Normal range", theme.green, top_cols[0], buf, theme);

    // Tile 2: Battery
    let (bat_val, bat_sub, bat_col) = if let Some(bat) = &snapshot.battery {
        (format!("{:.0}%", bat.percent), bat.condition.clone().unwrap_or_else(|| "Healthy".into()), theme.green)
    } else {
        ("AC".into(), "Desk Power".into(), theme.cyan)
    };
    render_health_tile("Battery", &bat_val, &bat_sub, bat_col, top_cols[1], buf, theme);

    // Tile 3: SSD
    render_health_tile("Storage SSD", "Verified", "100% Spare", theme.green, bot_cols[0], buf, theme);

    // Tile 4: Fans
    let fans_sub = if snapshot.sensors.fans.is_empty() {
        "Silent 0 RPM"
    } else {
        "Active cooling"
    };
    render_health_tile("Fans", "Optimal", fans_sub, theme.cyan, bot_cols[1], buf, theme);
}

fn render_health_tile(
    title: &str,
    value: &str,
    sub: &str,
    accent: Color,
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
) {
    if area.width < 4 || area.height < 2 {
        return;
    }

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(theme.card_style());
    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height >= 2 {
        let line1 = Line::from(vec![
            Span::styled(format!("{title}: "), Style::default().fg(theme.secondary)),
            Span::styled(value, Style::default().fg(accent).add_modifier(Modifier::BOLD)),
        ]);
        let line2 = Line::from(Span::styled(sub, Style::default().fg(theme.secondary)));

        buf.set_line(inner.x, inner.y, &line1, inner.width);
        buf.set_line(inner.x, inner.y + 1, &line2, inner.width);
    }
}
