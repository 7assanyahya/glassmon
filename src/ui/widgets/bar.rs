//! Custom bar widgets using eighth blocks (`▏▎▍▌▋▊▉█`) and stacked bars.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

const EIGHTHS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

/// Renders a horizontal eighth-block bar.
pub struct SmoothBar<'a> {
    pub ratio: f64, // 0.0..=1.0
    pub fg: Color,
    pub track_bg: Color,
    pub label: Option<&'a str>,
}

impl<'a> SmoothBar<'a> {
    pub fn new(ratio: f64, fg: Color, track_bg: Color) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            fg,
            track_bg,
            label: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl Widget for SmoothBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = area.width as usize;
        let total_eighths = (self.ratio * width as f64 * 8.0).round() as usize;
        let full_blocks = total_eighths / 8;
        let rem_eighth = total_eighths % 8;

        for x in 0..width {
            let cell = buf.cell_mut((area.x + x as u16, area.y));
            if let Some(cell) = cell {
                if x < full_blocks {
                    cell.set_char('█');
                    cell.set_fg(self.fg);
                    cell.set_bg(self.track_bg);
                } else if x == full_blocks && rem_eighth > 0 {
                    cell.set_char(EIGHTHS[rem_eighth - 1]);
                    cell.set_fg(self.fg);
                    cell.set_bg(self.track_bg);
                } else {
                    cell.set_char(' ');
                    cell.set_bg(self.track_bg);
                }
            }
        }

        if let Some(lbl) = self.label {
            let chars: Vec<char> = lbl.chars().collect();
            let start = area.x + (area.width.saturating_sub(chars.len() as u16)) / 2;
            for (i, &ch) in chars.iter().enumerate() {
                let pos_x = start + i as u16;
                if pos_x < area.x + area.width
                    && let Some(cell) = buf.cell_mut((pos_x, area.y))
                {
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(Color::White));
                }
            }
        }
    }
}

/// Largest remainder algorithm to apportion integer widths proportional to values.
pub fn largest_remainder(values: &[u64], total_width: usize) -> Vec<usize> {
    if values.is_empty() || total_width == 0 {
        return vec![0; values.len()];
    }

    let sum: u64 = values.iter().sum();
    if sum == 0 {
        return vec![0; values.len()];
    }

    let mut floors = Vec::with_capacity(values.len());
    let mut remainders = Vec::with_capacity(values.len());
    let mut assigned = 0;

    for (i, &v) in values.iter().enumerate() {
        let exact = (v as f64 / sum as f64) * total_width as f64;
        let floor = exact.floor() as usize;
        floors.push(floor);
        remainders.push((exact - floor as f64, i));
        assigned += floor;
    }

    // Distribute remaining slots to the highest fractional remainders
    remainders.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let diff = total_width.saturating_sub(assigned);
    for (_, idx) in remainders.into_iter().take(diff) {
        floors[idx] += 1;
    }

    floors
}

/// Stacked segment for a multi-category bar.
pub struct Segment<'a> {
    pub label: &'a str,
    pub value: u64,
    pub color: Color,
}

pub struct StackedBar<'a> {
    pub segments: Vec<Segment<'a>>,
    pub empty_color: Color,
}

impl<'a> StackedBar<'a> {
    pub fn new(segments: Vec<Segment<'a>>, empty_color: Color) -> Self {
        Self {
            segments,
            empty_color,
        }
    }
}

impl Widget for StackedBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let values: Vec<u64> = self.segments.iter().map(|s| s.value).collect();
        let widths = largest_remainder(&values, area.width as usize);

        let mut curr_x = area.x;
        for (seg, &w) in self.segments.iter().zip(widths.iter()) {
            for x in 0..(w as u16) {
                if curr_x + x < area.x + area.width
                    && let Some(cell) = buf.cell_mut((curr_x + x, area.y))
                {
                    cell.set_char('█');
                    cell.set_fg(seg.color);
                    cell.set_bg(self.empty_color);
                }
            }
            curr_x += w as u16;
        }

        // Fill any leftover if sum was 0
        while curr_x < area.x + area.width {
            if let Some(cell) = buf.cell_mut((curr_x, area.y)) {
                cell.set_char(' ');
                cell.set_bg(self.empty_color);
            }
            curr_x += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_largest_remainder_sums_to_exact_width() {
        let vals = [100, 200, 300, 400];
        let widths = largest_remainder(&vals, 50);
        assert_eq!(widths.iter().sum::<usize>(), 50);
        assert_eq!(widths, vec![5, 10, 15, 20]);

        let odd_vals = [33, 33, 34];
        let odd_widths = largest_remainder(&odd_vals, 10);
        assert_eq!(odd_widths.iter().sum::<usize>(), 10);
    }

    #[test]
    fn test_largest_remainder_zero_sum() {
        let widths = largest_remainder(&[0, 0, 0], 20);
        assert_eq!(widths, vec![0, 0, 0]);
    }
}
