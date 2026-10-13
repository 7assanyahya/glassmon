//! Rounded card widget container with title and key badge.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::ui::theme::Theme;

pub struct Card<'a> {
    title: &'a str,
    key_badge: Option<&'a str>,
    theme: &'a Theme,
    border_color: Option<Color>,
}

impl<'a> Card<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            title,
            key_badge: None,
            theme,
            border_color: None,
        }
    }

    pub fn key_badge(mut self, badge: &'a str) -> Self {
        self.key_badge = Some(badge);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn block(&self) -> Block<'a> {
        let mut title_spans = vec![
            Span::raw(" "),
            Span::styled(
                self.title,
                Style::default()
                    .fg(self.theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if let Some(badge) = self.key_badge {
            title_spans.push(Span::raw(" ["));
            title_spans.push(Span::styled(
                badge,
                Style::default()
                    .fg(self.theme.cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            title_spans.push(Span::raw("]"));
        }
        title_spans.push(Span::raw(" "));

        let border_fg = self.border_color.unwrap_or(self.theme.border);

        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_fg))
            .style(self.theme.card_style())
            .title(Line::from(title_spans))
    }
}

impl Widget for Card<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block().render(area, buf);
    }
}
