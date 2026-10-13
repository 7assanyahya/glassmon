//! Network page rendering with per-interface breakdown, totals and scrolling sparklines.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, Widget};

use crate::app::App;
use crate::model::Snapshot;
use crate::ui::widgets::{Card, Sparkline};
use crate::units;

pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // Interfaces table
            Constraint::Percentage(55), // Graphs and selected interface details
        ])
        .split(area);

    render_interface_table(app, snapshot, chunks[0], buf);
    render_interface_details(app, snapshot, chunks[1], buf);
}

fn render_interface_table(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Network Interfaces", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    let header_cells = ["INTERFACE", "TYPE", "DOWN RATE", "UP RATE", "TOTAL DOWN", "TOTAL UP", "MAC"]
        .into_iter()
        .map(|h| Cell::from(Span::styled(h, Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD))));
    let header = Row::new(header_cells).height(1);

    let ifaces = &snapshot.network.interfaces;
    let selected_idx = if ifaces.is_empty() { 0 } else { app.net_iface_index % ifaces.len() };

    let rows: Vec<Row> = ifaces
        .iter()
        .enumerate()
        .map(|(idx, iface)| {
            let is_selected = idx == selected_idx;
            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default().fg(theme.text)
            };

            let down_col = if is_selected { Color::White } else { theme.green };
            let up_col = if is_selected { Color::White } else { theme.orange };

            Row::new(vec![
                Cell::from(Span::styled(&iface.name, Style::default().add_modifier(Modifier::BOLD))),
                Cell::from(iface.kind.label()),
                Cell::from(Span::styled(units::rate(iface.rx_rate), Style::default().fg(down_col))),
                Cell::from(Span::styled(units::rate(iface.tx_rate), Style::default().fg(up_col))),
                Cell::from(units::bytes(iface.rx_total)),
                Cell::from(units::bytes(iface.tx_total)),
                Cell::from(iface.mac.as_deref().unwrap_or("-")),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Min(18),
    ];

    let table = Table::new(rows, widths).header(header);
    Widget::render(table, inner, buf);
}

fn render_interface_details(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let ifaces = &snapshot.network.interfaces;
    if ifaces.is_empty() {
        return;
    }

    let selected_idx = app.net_iface_index % ifaces.len();
    let iface = &ifaces[selected_idx];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left card: Download history
    let dl_title = format!("Download (RX) - {}", iface.name);
    let dl_card = Card::new(&dl_title, theme);
    let dl_inner = dl_card.block().inner(chunks[0]);
    dl_card.render(chunks[0], buf);

    if dl_inner.height >= 2 {
        let rx_u64: Vec<u64> = iface.rx_history.iter().map(|&v| (v / 1024.0) as u64).collect();
        let subchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(dl_inner);

        let dl_text = Line::from(vec![
            Span::styled("Current Rate: ", Style::default().fg(theme.secondary)),
            Span::styled(units::rate(iface.rx_rate), Style::default().fg(theme.green).add_modifier(Modifier::BOLD)),
            Span::styled("   Total: ", Style::default().fg(theme.secondary)),
            Span::styled(units::bytes(iface.rx_total), Style::default().fg(theme.text)),
        ]);
        buf.set_line(subchunks[0].x, subchunks[0].y, &dl_text, subchunks[0].width);
        Sparkline::new(&rx_u64, theme.green).render(subchunks[1], buf);
    }

    // Right card: Upload history
    let ul_title = format!("Upload (TX) - {}", iface.name);
    let ul_card = Card::new(&ul_title, theme);
    let ul_inner = ul_card.block().inner(chunks[1]);
    ul_card.render(chunks[1], buf);

    if ul_inner.height >= 2 {
        let tx_u64: Vec<u64> = iface.tx_history.iter().map(|&v| (v / 1024.0) as u64).collect();
        let subchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(ul_inner);

        let ul_text = Line::from(vec![
            Span::styled("Current Rate: ", Style::default().fg(theme.secondary)),
            Span::styled(units::rate(iface.tx_rate), Style::default().fg(theme.orange).add_modifier(Modifier::BOLD)),
            Span::styled("   Total: ", Style::default().fg(theme.secondary)),
            Span::styled(units::bytes(iface.tx_total), Style::default().fg(theme.text)),
        ]);
        buf.set_line(subchunks[0].x, subchunks[0].y, &ul_text, subchunks[0].width);
        Sparkline::new(&tx_u64, theme.orange).render(subchunks[1], buf);
    }
}
