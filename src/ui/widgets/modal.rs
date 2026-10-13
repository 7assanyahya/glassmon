//! Modals for confirmation (process kill, eject volume) and keybindings help overlay.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Widget};

use crate::ui::theme::Theme;

pub enum ModalType {
    ConfirmKill {
        pid: u32,
        name: String,
        escalate_sigkill: bool,
    },
    ConfirmEject {
        mount: String,
        device: String,
    },
    Help,
}

pub struct Modal<'a> {
    pub modal_type: &'a ModalType,
    pub theme: &'a Theme,
}

impl<'a> Modal<'a> {
    pub fn new(modal_type: &'a ModalType, theme: &'a Theme) -> Self {
        Self { modal_type, theme }
    }
}

impl Widget for Modal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.modal_type {
            ModalType::ConfirmKill {
                pid,
                name,
                escalate_sigkill,
            } => {
                let w = 56.min(area.width.saturating_sub(4));
                let h = 10.min(area.height.saturating_sub(4));
                let modal_area = centered_rect(w, h, area);
                Clear.render(modal_area, buf);

                let sig_label = if *escalate_sigkill {
                    ("SIGKILL (Force Kill)", self.theme.red)
                } else {
                    ("SIGTERM (Graceful Terminate)", self.theme.orange)
                };

                let block = Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(sig_label.1))
                    .style(self.theme.card_style())
                    .title(Line::from(vec![
                        Span::raw(" "),
                        Span::styled(
                            " Confirm Terminate Process ",
                            Style::default()
                                .fg(sig_label.1)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                    ]));

                let lines = vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("  Process: "),
                        Span::styled(
                            format!("{name} (PID {pid})"),
                            Style::default()
                                .fg(self.theme.text)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  Signal:  "),
                        Span::styled(
                            sig_label.0,
                            Style::default()
                                .fg(sig_label.1)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  [Enter] ", Style::default().fg(self.theme.cyan).add_modifier(Modifier::BOLD)),
                        Span::raw("Confirm   "),
                        Span::styled("  [Shift+K] ", Style::default().fg(self.theme.red).add_modifier(Modifier::BOLD)),
                        Span::raw("Switch to SIGKILL   "),
                        Span::styled("  [Esc] ", Style::default().fg(self.theme.secondary).add_modifier(Modifier::BOLD)),
                        Span::raw("Cancel"),
                    ]),
                ];

                Paragraph::new(lines).block(block).render(modal_area, buf);
            }
            ModalType::ConfirmEject { mount, device } => {
                let w = 54.min(area.width.saturating_sub(4));
                let h = 9.min(area.height.saturating_sub(4));
                let modal_area = centered_rect(w, h, area);
                Clear.render(modal_area, buf);

                let block = Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.theme.orange))
                    .style(self.theme.card_style())
                    .title(" Confirm Unmount / Eject ");

                let lines = vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("  Mount:  "),
                        Span::styled(
                            mount.as_str(),
                            Style::default()
                                .fg(self.theme.text)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("  Device: "),
                        Span::styled(device.as_str(), Style::default().fg(self.theme.secondary)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  [Enter] ", Style::default().fg(self.theme.cyan).add_modifier(Modifier::BOLD)),
                        Span::raw("Unmount   "),
                        Span::styled("  [Esc] ", Style::default().fg(self.theme.secondary).add_modifier(Modifier::BOLD)),
                        Span::raw("Cancel"),
                    ]),
                ];

                Paragraph::new(lines).block(block).render(modal_area, buf);
            }
            ModalType::Help => {
                let w = 72.min(area.width.saturating_sub(2));
                let h = 24.min(area.height.saturating_sub(2));
                let modal_area = centered_rect(w, h, area);
                Clear.render(modal_area, buf);

                let block = Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(self.theme.blue))
                    .style(self.theme.card_style())
                    .title(Line::from(vec![
                        Span::raw(" "),
                        Span::styled(
                            " Keyboard Shortcuts & Navigation ",
                            Style::default()
                                .fg(self.theme.blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                    ]));

                let help_entries = [
                    ("Tab / Shift+Tab", "Next / Previous page"),
                    ("1 - 6", "Jump to page (Overview, Procs, Net, Disks, HW, Device)"),
                    ("t", "Toggle Dark / Light theme"),
                    ("p", "Pause / resume sampling"),
                    ("q / Ctrl+C", "Quit application"),
                    ("?", "Toggle this help overlay"),
                    ("j / k / ↑ / ↓", "Move row selection up / down"),
                    ("g / G / PgUp / PgDn", "Jump to start / end / page down"),
                    ("s", "Cycle sort column (CPU, Mem, PID, Name)"),
                    ("r", "Reverse sort order / Rescan ports"),
                    ("/", "Start search filter (case-insensitive)"),
                    ("Esc", "Clear search filter / close modal"),
                    ("K", "Terminate selected process (SIGTERM)"),
                    ("Shift+K (in modal)", "Escalate terminate to SIGKILL"),
                    ("n", "Cycle selected network interface"),
                    ("h / l", "Switch storage drive / volume"),
                    ("e", "Eject / unmount selected volume"),
                    ("u", "Toggle Celsius / Fahrenheit"),
                    ("Mouse click / scroll", "Click tabs, scroll lists, select rows"),
                ];

                let mut lines = Vec::new();
                lines.push(Line::from(""));
                for (key, desc) in help_entries {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {key:<22}"),
                            Style::default()
                                .fg(self.theme.cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(desc, Style::default().fg(self.theme.text)),
                    ]));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  Press [Esc] or [?] to close",
                        Style::default().fg(self.theme.secondary),
                    ),
                ]));

                Paragraph::new(lines).block(block).render(modal_area, buf);
            }
        }
    }
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}
