//! Top-level UI orchestration, shell navigation, responsive breakpoints, modals and toasts.

pub mod pages;
pub mod theme;
pub mod widgets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::app::{App, Page};
use crate::model::Snapshot;
use crate::ui::widgets::{Keycap, Modal, TabBar, ToastWidget};

/// Minimum terminal dimensions required for glassmon.
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

pub fn draw(f: &mut Frame, app: &App, snapshot: &Snapshot) {
    let area = f.area();

    // Check minimum terminal size constraint
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(f, app, area);
        return;
    }

    // Main layout:
    // 0: Tab bar (1 line)
    // 1: Page title & subtitle (2 lines)
    // 2: Page contents (Min(15))
    // 3: Footer key hints (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab Bar
            Constraint::Length(2), // Title & Subtitle
            Constraint::Min(15),   // Main page area
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // 1. Tab Bar
    let tab_bar = TabBar::new(app.page, &app.theme, app.paused);
    f.render_widget(tab_bar, chunks[0]);

    // 2. Title & Subtitle
    render_header(f, app, chunks[1]);

    // 3. Page Body
    let body_area = chunks[2];
    match app.page {
        Page::Overview => pages::overview::render(app, snapshot, body_area, f.buffer_mut()),
        Page::Processes => pages::processes::render(app, snapshot, body_area, f.buffer_mut()),
        Page::Network => pages::network::render(app, snapshot, body_area, f.buffer_mut()),
        Page::Disks => pages::disks::render(app, snapshot, body_area, f.buffer_mut()),
        Page::Hardware => pages::hardware::render(app, snapshot, body_area, f.buffer_mut()),
        Page::Device => pages::device::render(app, snapshot, body_area, f.buffer_mut()),
    }

    // 4. Footer
    render_footer(f, app, chunks[3]);

    // 5. Active Modal (if open)
    if let Some(modal) = &app.active_modal {
        let m = Modal::new(modal, &app.theme);
        f.render_widget(m, area);
    }

    // 6. Toasts (if active)
    for toast in &app.toasts {
        let w = ToastWidget {
            toast,
            theme: &app.theme,
        };
        f.render_widget(w, area);
    }
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let title_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            app.page.title(),
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let subtitle_line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            app.page.subtitle(),
            Style::default().fg(theme.secondary),
        ),
    ]);

    let paragraph = Paragraph::new(vec![title_line, subtitle_line]);
    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut spans = Vec::new();

    let keys = [
        ("Tab", "Page"),
        ("t", "Theme"),
        ("p", "Pause"),
        ("?", "Help"),
        ("q", "Quit"),
    ];

    for (k, desc) in keys {
        spans.extend(Keycap::render(k, desc, theme));
    }

    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}

fn render_too_small(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let card_w = 44.min(area.width);
    let card_h = 7.min(area.height);

    let x = (area.width.saturating_sub(card_w)) / 2;
    let y = (area.height.saturating_sub(card_h)) / 2;
    let center_rect = Rect {
        x,
        y,
        width: card_w,
        height: card_h,
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.red))
        .style(theme.card_style())
        .title(" Terminal Too Small ");

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Required: "),
            Span::styled(
                format!("{MIN_WIDTH} x {MIN_HEIGHT}"),
                Style::default().fg(theme.cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Current:  "),
            Span::styled(
                format!("{} x {}", area.width, area.height),
                Style::default().fg(theme.orange).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), center_rect);
}
