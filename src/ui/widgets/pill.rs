//! Status pill and badge widgets.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

pub struct Pill;

impl Pill {
    /// Renders a small filled status chip: ` ● Connected ` or ` Verified `
    pub fn chip<'a>(text: &'a str, fg: Color, bg: Color) -> Vec<Span<'a>> {
        vec![
            Span::styled(" ", Style::default().bg(bg)),
            Span::styled(
                text,
                Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default().bg(bg)),
        ]
    }

    /// Renders a status dot with text: `● 40 Gb/s`
    pub fn dot_text<'a>(text: &'a str, dot_color: Color, text_color: Color) -> Vec<Span<'a>> {
        vec![
            Span::styled("● ", Style::default().fg(dot_color)),
            Span::styled(text, Style::default().fg(text_color)),
        ]
    }
}
