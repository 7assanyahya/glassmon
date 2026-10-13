//! Snapshot and rendering tests using ratatui's `TestBackend`.
//!
//! Validates that all six pages render without crashing, under both
//! 120x36 (standard target) and 80x24 (minimum responsive target),
//! in both Dark and Light themes and Monochrome mode.

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use glassmon::app::{App, Page};
use glassmon::data::Collector;
use glassmon::data::demo::DemoCollector;
use glassmon::model::Needs;
use glassmon::ui;
use glassmon::ui::theme::{ColorCapability, ThemeMode};

#[test]
fn test_render_all_pages_120x36() {
    let mut collector = DemoCollector::new();
    let snapshot = collector.sample(&Needs::all());

    let pages = [
        Page::Overview,
        Page::Processes,
        Page::Network,
        Page::Disks,
        Page::Hardware,
        Page::Device,
    ];

    for page in pages {
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut app = App::new(ThemeMode::Dark, ColorCapability::TrueColor);
        app.page = page;

        terminal
            .draw(|f| {
                ui::draw(f, &app, &snapshot);
            })
            .expect("Failed to draw page at 120x36");

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer.area.width, 120);
        assert_eq!(buffer.area.height, 36);
    }
}

#[test]
fn test_render_all_pages_80x24() {
    let mut collector = DemoCollector::new();
    let snapshot = collector.sample(&Needs::all());

    let pages = [
        Page::Overview,
        Page::Processes,
        Page::Network,
        Page::Disks,
        Page::Hardware,
        Page::Device,
    ];

    for page in pages {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut app = App::new(ThemeMode::Light, ColorCapability::Color256);
        app.page = page;

        terminal
            .draw(|f| {
                ui::draw(f, &app, &snapshot);
            })
            .expect("Failed to draw page at 80x24");

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer.area.width, 80);
        assert_eq!(buffer.area.height, 24);
    }
}

#[test]
fn test_render_too_small_breakpoint() {
    let mut collector = DemoCollector::new();
    let snapshot = collector.sample(&Needs::all());

    let backend = TestBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let app = App::new(ThemeMode::Dark, ColorCapability::Monochrome);

    terminal
        .draw(|f| {
            ui::draw(f, &app, &snapshot);
        })
        .expect("Failed to draw small terminal warning");

    // Ensure it rendered warning text
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();

    assert!(content.contains("Terminal Too Small"));
    assert!(content.contains("Required: 80 x 24"));
}
