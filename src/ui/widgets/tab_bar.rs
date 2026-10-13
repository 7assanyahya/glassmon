//! Segmented tab bar widget with active pill and clock.

use chrono::Local;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::Page;
use crate::ui::theme::Theme;

pub struct TabBar<'a> {
    pub current: Page,
    pub theme: &'a Theme,
    pub paused: bool,
}

impl<'a> TabBar<'a> {
    pub fn new(current: Page, theme: &'a Theme, paused: bool) -> Self {
        Self {
            current,
            theme,
            paused,
        }
    }
}

impl Widget for TabBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let mut spans = Vec::new();
        spans.push(Span::styled(" ", Style::default()));

        let pages = [
            (Page::Overview, "1", "Overview"),
            (Page::Processes, "2", "Processes"),
            (Page::Network, "3", "Network"),
            (Page::Disks, "4", "Disks"),
            (Page::Hardware, "5", "Hardware"),
            (Page::Device, "6", "Device"),
        ];

        for (page, num, name) in pages {
            let is_active = self.current == page;
            if is_active {
                spans.push(Span::styled(
                    format!(" {num} {name} "),
                    Style::default()
                        .bg(self.theme.blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    format!(" {num} "),
                    Style::default().fg(self.theme.cyan),
                ));
                spans.push(Span::styled(
                    format!("{name} "),
                    Style::default().fg(self.theme.secondary),
                ));
            }
            spans.push(Span::raw(" "));
        }

        if self.paused {
            spans.push(Span::styled(
                " [PAUSED] ",
                Style::default()
                    .bg(self.theme.orange)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Render tabs on left
        let left_line = Line::from(spans);
        buf.set_line(area.x, area.y, &left_line, area.width);

        // Render clock on right
        let now = Local::now().format("%H:%M:%S").to_string();
        let clock_span = Span::styled(
            format!("{now} "),
            Style::default().fg(self.theme.secondary),
        );
        let clock_len = now.len() as u16 + 1;
        if area.width > clock_len {
            let clock_x = area.x + area.width - clock_len;
            buf.set_span(clock_x, area.y, &clock_span, clock_len);
        }
    }
}
