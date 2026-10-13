//! Status toast notifications.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};
use std::time::Instant;

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration_secs: u64,
}

impl Toast {
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
            created_at: Instant::now(),
            duration_secs: 3,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() >= self.duration_secs
    }
}

pub struct ToastWidget<'a> {
    pub toast: &'a Toast,
    pub theme: &'a Theme,
}

impl Widget for ToastWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (border_color, prefix) = match self.toast.kind {
            ToastKind::Info => (self.theme.blue, "ℹ "),
            ToastKind::Success => (self.theme.green, "✓ "),
            ToastKind::Warning => (self.theme.orange, "⚠ "),
            ToastKind::Error => (self.theme.red, "✕ "),
        };

        let msg_len = self.toast.message.chars().count() as u16 + 6;
        let width = msg_len.min(area.width.saturating_sub(4)).max(20);
        let height = 3;

        let x = area.x + area.width.saturating_sub(width + 2);
        let y = area.y + 2;
        let toast_area = Rect {
            x,
            y,
            width,
            height,
        };

        Clear.render(toast_area, buf);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(self.theme.card_style());

        let line = Line::from(vec![
            Span::styled(
                prefix,
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&self.toast.message, Style::default().fg(self.theme.text)),
        ]);

        Paragraph::new(line).block(block).render(toast_area, buf);
    }
}
