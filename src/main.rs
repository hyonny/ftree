mod app;
mod clipboard;
mod input;
mod search;
mod tree;
mod ui;

use std::io;

use app::App;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new()?;

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if input::handle_key(&mut app, key.code) {
                    break;
                }
            }
        }
    }

    Ok(())
}
