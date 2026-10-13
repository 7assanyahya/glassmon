use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::{Block, BorderType, Paragraph};

fn main() -> Result<()> {
    let (mut terminal, _guard) = glassmon::tui::init(false)?;
    loop {
        terminal.draw(|f| {
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" glassmon ");
            f.render_widget(Paragraph::new("press q to quit").block(block), f.area());
        })?;
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            let ctrl_c = k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL);
            if k.code == KeyCode::Char('q') || ctrl_c {
                break;
            }
        }
    }
    Ok(())
}
