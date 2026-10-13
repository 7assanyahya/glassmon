//! Sparkline wrapper with automatic scaling and title.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Sparkline as RataSparkline, Widget};

pub struct Sparkline<'a> {
    data: &'a [u64],
    color: Color,
    max: Option<u64>,
}

impl<'a> Sparkline<'a> {
    pub fn new(data: &'a [u64], color: Color) -> Self {
        Self {
            data,
            color,
            max: None,
        }
    }

    pub fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }
}

impl Widget for Sparkline<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 || self.data.is_empty() {
            return;
        }

        let mut widget = RataSparkline::default()
            .data(self.data)
            .style(ratatui::style::Style::default().fg(self.color));

        if let Some(m) = self.max {
            widget = widget.max(m);
        }

        widget.render(area, buf);
    }
}
