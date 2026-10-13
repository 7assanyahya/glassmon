//! Full interactive processes monitor table with sorting, filtering and signal dispatch.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, Widget};

use crate::app::{App, ProcessSort};
use crate::model::Snapshot;
use crate::ui::widgets::Card;
use crate::units;

pub fn render(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter / stats bar
            Constraint::Min(4),    // Process table
        ])
        .split(area);

    render_filter_bar(app, snapshot, chunks[0], buf);
    render_table(app, snapshot, chunks[1], buf);
}

fn render_filter_bar(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let mut spans = Vec::new();

    spans.push(Span::styled(
        " Total: ",
        Style::default().fg(theme.secondary),
    ));
    spans.push(Span::styled(
        format!(
            "{} ",
            snapshot.host.process_count.max(snapshot.processes.len())
        ),
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled(
        " Sort: ",
        Style::default().fg(theme.secondary),
    ));
    let sort_dir = if app.process_sort_reverse {
        "▲"
    } else {
        "▼"
    };
    spans.push(Span::styled(
        format!("{} {sort_dir} ", app.process_sort.label()),
        Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD),
    ));

    if app.filter_active || !app.process_filter.is_empty() {
        spans.push(Span::styled(
            " Filter: ",
            Style::default()
                .fg(theme.orange)
                .add_modifier(Modifier::BOLD),
        ));
        let cursor = if app.filter_active { "█" } else { "" };
        spans.push(Span::styled(
            format!("\"{}\"{cursor} ", app.process_filter),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            " (Press [/] to filter, [s] to sort, [K] to kill) ",
            Style::default().fg(theme.secondary),
        ));
    }

    buf.set_line(area.x, area.y, &Line::from(spans), area.width);
}

fn render_table(app: &App, snapshot: &Snapshot, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let card = Card::new("Processes", theme);
    let inner = card.block().inner(area);
    card.render(area, buf);

    if inner.height < 2 || inner.width < 10 {
        return;
    }

    // Filter and sort processes
    let mut procs: Vec<_> = snapshot
        .processes
        .iter()
        .filter(|p| {
            if app.process_filter.is_empty() {
                true
            } else {
                let q = app.process_filter.to_lowercase();
                p.name.to_lowercase().contains(&q)
                    || p.command.to_lowercase().contains(&q)
                    || p.user.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || p.pid.to_string().contains(&q)
            }
        })
        .collect();

    match app.process_sort {
        ProcessSort::Cpu => procs.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        ProcessSort::Memory => procs.sort_by(|a, b| b.memory.cmp(&a.memory)),
        ProcessSort::Pid => procs.sort_by(|a, b| a.pid.cmp(&b.pid)),
        ProcessSort::Name => {
            procs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
    }

    if app.process_sort_reverse {
        procs.reverse();
    }

    let header_titles = [
        ("PID", 8),
        ("NAME", 20),
        ("USER", 12),
        ("CPU%", 8),
        ("MEM", 10),
        ("THREADS", 9),
        ("STATE", 7),
        ("COMMAND", 30),
    ];

    let header_cells = header_titles.iter().map(|(h, _)| {
        let is_sorted = match app.process_sort {
            ProcessSort::Cpu => *h == "CPU%",
            ProcessSort::Memory => *h == "MEM",
            ProcessSort::Pid => *h == "PID",
            ProcessSort::Name => *h == "NAME",
        };
        let style = if is_sorted {
            Style::default()
                .fg(theme.cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD)
        };
        Cell::from(Span::styled(*h, style))
    });
    let header = Row::new(header_cells).height(1);

    let max_visible = inner.height.saturating_sub(1) as usize;
    let selected_idx = app.process_selected.min(procs.len().saturating_sub(1));

    // Calculate scroll offset so selected row stays in view
    let offset = if selected_idx >= max_visible {
        selected_idx - max_visible + 1
    } else {
        0
    };

    let rows: Vec<Row> = procs
        .iter()
        .skip(offset)
        .take(max_visible)
        .enumerate()
        .map(|(rel_idx, p)| {
            let abs_idx = offset + rel_idx;
            let is_selected = abs_idx == selected_idx;

            let row_style = if is_selected {
                theme.selected_row()
            } else {
                Style::default().fg(theme.text)
            };

            let cpu_col = if is_selected {
                Color::White
            } else {
                theme.usage_color(p.cpu as f64)
            };

            let user_str = p.user.as_deref().unwrap_or("-");
            let threads_str = p
                .threads
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".into());

            Row::new(vec![
                Cell::from(format!("{}", p.pid)),
                Cell::from(units::truncate(&p.name, 18)),
                Cell::from(units::truncate(user_str, 10)),
                Cell::from(Span::styled(
                    format!("{:.1}%", p.cpu),
                    Style::default().fg(cpu_col),
                )),
                Cell::from(units::bytes(p.memory)),
                Cell::from(threads_str),
                Cell::from(p.state.clone()),
                Cell::from(units::truncate(&p.command, 40)),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(20),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(9),
        Constraint::Length(7),
        Constraint::Min(25),
    ];

    let table = Table::new(rows, widths).header(header);
    Widget::render(table, inner, buf);
}
