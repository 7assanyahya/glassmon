//! Terminal setup and teardown.
//!
//! The terminal is restored on every exit path: normal return, `?` errors
//! (via [`TerminalGuard`]'s `Drop`), Ctrl+C (handled as a key event in raw
//! mode) and panics (via a panic hook installed before raw mode is enabled).
//! [`restore`] is idempotent, so calling it from several of these paths is
//! harmless.

use std::io::{self, Stdout, stdout};
use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::cursor::Show;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

/// The concrete terminal type used by the app.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static MOUSE: AtomicBool = AtomicBool::new(false);
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Restores the terminal when dropped.
pub struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore();
    }
}

/// Puts the terminal into raw mode on the alternate screen and returns it
/// together with a guard that restores it on drop.
pub fn init(mouse: bool) -> io::Result<(Tui, TerminalGuard)> {
    install_panic_hook();
    enable_raw_mode()?;
    ACTIVE.store(true, Ordering::SeqCst);
    let guard = TerminalGuard;

    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    if mouse {
        execute!(out, EnableMouseCapture)?;
        MOUSE.store(true, Ordering::SeqCst);
    }
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok((terminal, guard))
}

/// Best-effort, idempotent terminal restore. Errors are ignored because this
/// runs on teardown paths (including panics) where nothing better can be done.
pub fn restore() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut out = stdout();
    if MOUSE.swap(false, Ordering::SeqCst) {
        let _ = execute!(out, DisableMouseCapture);
    }
    let _ = execute!(out, LeaveAlternateScreen, Show);
    let _ = disable_raw_mode();
}

fn install_panic_hook() {
    if HOOK_INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Restore first so the panic message is printed on a sane terminal.
        restore();
        previous(info);
    }));
}
