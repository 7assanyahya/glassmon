//! Keycap chip widgets for footers and shortcuts.

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use crate::ui::theme::Theme;

pub struct Keycap;

impl Keycap {
    /// Renders a keycap chip: `[Key] Action`
    pub fn render<'a>(key: &'a str, label: &'a str, theme: &'a Theme) -> Vec<Span<'a>> {
        vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                format!(" {key} "),
                Style::default()
                    .bg(theme.empty_track)
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {label} "),
                Style::default().fg(theme.secondary),
            ),
        ]
    }
}
