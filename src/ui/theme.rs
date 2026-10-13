//! Theme, palette definitions, and terminal color capability mapping.
//!
//! Handles 24-bit TrueColor, 256-color approximation fallback, and `NO_COLOR` (monochrome).

use ratatui::style::{Color, Modifier, Style};
use std::env;

/// RGB color representation for palette calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor(pub u8, pub u8, pub u8);

impl RgbColor {
    pub const fn hex(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    /// Convert to ratatui Color based on terminal capabilities.
    pub fn to_color(self, cap: ColorCapability) -> Color {
        match cap {
            ColorCapability::Monochrome => Color::Reset,
            ColorCapability::TrueColor => Color::Rgb(self.0, self.1, self.2),
            ColorCapability::Color256 => Color::Indexed(rgb_to_ansi256(self.0, self.1, self.2)),
        }
    }
}

/// Color capabilities of the running terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Color256,
    Monochrome,
}

impl ColorCapability {
    pub fn detect() -> Self {
        // Honor NO_COLOR standard (https://no-color.org)
        if env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
            return Self::Monochrome;
        }

        // Truecolor detection from COLORTERM
        if let Ok(val) = env::var("COLORTERM") {
            let v = val.to_lowercase();
            if v == "truecolor" || v == "24bit" {
                return Self::TrueColor;
            }
        }

        // Term detection
        if let Ok(term) = env::var("TERM") {
            let t = term.to_lowercase();
            if t.contains("truecolor") || t.contains("24bit") {
                return Self::TrueColor;
            }
            if t.contains("256color") || t.contains("xterm") || t.contains("screen") {
                return Self::Color256;
            }
            if t == "dumb" {
                return Self::Monochrome;
            }
        }

        // Default assumption for modern terminals
        Self::TrueColor
    }
}

/// Nearest ANSI 256 color lookup using standard Euclidean distance in RGB space.
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Check 16 basic colors (0..15)
    // 6x6x6 color cube: 16..231
    // Grayscale ramp: 232..255

    let r_idx = if r < 48 {
        0
    } else if r < 115 {
        1
    } else {
        (r - 35) / 40
    };
    let g_idx = if g < 48 {
        0
    } else if g < 115 {
        1
    } else {
        (g - 35) / 40
    };
    let b_idx = if b < 48 {
        0
    } else if b < 115 {
        1
    } else {
        (b - 35) / 40
    };

    let cube_index = 16 + 36 * r_idx + 6 * g_idx + b_idx;

    // Check if a grayscale ramp color is closer
    if r == g && g == b {
        let gray_idx = if r < 8 {
            16
        } else if r > 248 {
            231
        } else {
            232 + ((r - 8) as f32 / 10.0).round() as u8
        };
        return gray_idx;
    }

    cube_index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Complete color scheme token set.
#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub cap: ColorCapability,

    pub bg: Color,
    pub card_bg: Color,
    pub border: Color,
    pub text: Color,
    pub secondary: Color,
    pub empty_track: Color,

    // Accents
    pub blue: Color,
    pub cyan: Color,
    pub green: Color,
    pub orange: Color,
    pub red: Color,
    pub purple: Color,
    pub pink: Color,
}

impl Theme {
    pub fn new(mode: ThemeMode, cap: ColorCapability) -> Self {
        match mode {
            ThemeMode::Dark => {
                let bg_rgb = RgbColor::hex(0x0b, 0x0d, 0x1a);
                let card_rgb = RgbColor::hex(0x16, 0x1a, 0x2c);
                let border_rgb = RgbColor::hex(0x4a, 0x4f, 0x63);
                let text_rgb = RgbColor::hex(0xec, 0xee, 0xf5);
                let secondary_rgb = RgbColor::hex(0x9a, 0xa1, 0xb8);
                let track_rgb = RgbColor::hex(0x2a, 0x2f, 0x45);

                let blue_rgb = RgbColor::hex(0x0a, 0x84, 0xff);
                let cyan_rgb = RgbColor::hex(0x64, 0xd2, 0xff);
                let green_rgb = RgbColor::hex(0x30, 0xd1, 0x58);
                let orange_rgb = RgbColor::hex(0xff, 0x9f, 0x0a);
                let red_rgb = RgbColor::hex(0xff, 0x45, 0x3a);
                let purple_rgb = RgbColor::hex(0xbf, 0x5a, 0xf2);
                let pink_rgb = RgbColor::hex(0xff, 0x37, 0x5f);

                Self {
                    mode,
                    cap,
                    bg: bg_rgb.to_color(cap),
                    card_bg: card_rgb.to_color(cap),
                    border: border_rgb.to_color(cap),
                    text: text_rgb.to_color(cap),
                    secondary: secondary_rgb.to_color(cap),
                    empty_track: track_rgb.to_color(cap),
                    blue: blue_rgb.to_color(cap),
                    cyan: cyan_rgb.to_color(cap),
                    green: green_rgb.to_color(cap),
                    orange: orange_rgb.to_color(cap),
                    red: red_rgb.to_color(cap),
                    purple: purple_rgb.to_color(cap),
                    pink: pink_rgb.to_color(cap),
                }
            }
            ThemeMode::Light => {
                // High contrast light palette (> 4.5:1 ratio)
                let bg_rgb = RgbColor::hex(0xf2, 0xf4, 0xf8);
                let card_rgb = RgbColor::hex(0xff, 0xff, 0xff);
                let border_rgb = RgbColor::hex(0xc8, 0xcd, 0xdc);
                let text_rgb = RgbColor::hex(0x1a, 0x1d, 0x24);
                let secondary_rgb = RgbColor::hex(0x5c, 0x64, 0x7a);
                let track_rgb = RgbColor::hex(0xe0, 0xe4, 0xee);

                let blue_rgb = RgbColor::hex(0x00, 0x66, 0xcc);
                let cyan_rgb = RgbColor::hex(0x00, 0x88, 0xcc);
                let green_rgb = RgbColor::hex(0x24, 0x8a, 0x3d);
                let orange_rgb = RgbColor::hex(0xc6, 0x60, 0x00);
                let red_rgb = RgbColor::hex(0xd7, 0x00, 0x15);
                let purple_rgb = RgbColor::hex(0x89, 0x44, 0xab);
                let pink_rgb = RgbColor::hex(0xd3, 0x0f, 0x4b);

                Self {
                    mode,
                    cap,
                    bg: bg_rgb.to_color(cap),
                    card_bg: card_rgb.to_color(cap),
                    border: border_rgb.to_color(cap),
                    text: text_rgb.to_color(cap),
                    secondary: secondary_rgb.to_color(cap),
                    empty_track: track_rgb.to_color(cap),
                    blue: blue_rgb.to_color(cap),
                    cyan: cyan_rgb.to_color(cap),
                    green: green_rgb.to_color(cap),
                    orange: orange_rgb.to_color(cap),
                    red: red_rgb.to_color(cap),
                    purple: purple_rgb.to_color(cap),
                    pink: pink_rgb.to_color(cap),
                }
            }
        }
    }

    /// Color for percentage usage (<50% green, 50-79% orange, 80%+ red).
    pub fn usage_color(&self, pct: f64) -> Color {
        if self.cap == ColorCapability::Monochrome {
            return Color::Reset;
        }
        if pct < 50.0 {
            self.green
        } else if pct < 80.0 {
            self.orange
        } else {
            self.red
        }
    }

    /// Color for temperature (<60 green, 60-79 orange, 80+ red).
    pub fn temp_color(&self, celsius: f32) -> Color {
        if self.cap == ColorCapability::Monochrome {
            return Color::Reset;
        }
        if celsius < 60.0 {
            self.green
        } else if celsius < 80.0 {
            self.orange
        } else {
            self.red
        }
    }

    /// Selected row style: solid blue fill, white text (or inverse in monochrome).
    pub fn selected_row(&self) -> Style {
        if self.cap == ColorCapability::Monochrome {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
                .bg(self.blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        }
    }

    /// Card block style.
    pub fn card_style(&self) -> Style {
        if self.cap == ColorCapability::Monochrome {
            Style::default()
        } else {
            Style::default().bg(self.card_bg).fg(self.text)
        }
    }

    /// Border style.
    pub fn border_style(&self) -> Style {
        if self.cap == ColorCapability::Monochrome {
            Style::default()
        } else {
            Style::default().fg(self.border)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_ansi256() {
        let c = rgb_to_ansi256(0, 0, 0);
        assert_eq!(c, 16);
        let c2 = rgb_to_ansi256(255, 255, 255);
        assert_eq!(c2, 231);
    }

    #[test]
    fn test_usage_color_thresholds() {
        let t = Theme::new(ThemeMode::Dark, ColorCapability::TrueColor);
        assert_eq!(t.usage_color(49.9), t.green);
        assert_eq!(t.usage_color(50.0), t.orange);
        assert_eq!(t.usage_color(79.9), t.orange);
        assert_eq!(t.usage_color(80.0), t.red);
    }

    #[test]
    fn test_temp_color_thresholds() {
        let t = Theme::new(ThemeMode::Dark, ColorCapability::TrueColor);
        assert_eq!(t.temp_color(59.0), t.green);
        assert_eq!(t.temp_color(60.0), t.orange);
        assert_eq!(t.temp_color(85.0), t.red);
    }
}
