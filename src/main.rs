mod app;
mod clipboard;
mod git;
mod input;
mod preview;
mod search;
mod tree;
mod ui;

use std::io;

use app::App;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    // マウスキャプチャを有効化
    execute!(io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal);
    // マウスキャプチャを無効化
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new()?;
    let mut mouse_captured = true;

    loop {
        // プレビュー表示中はマウスキャプチャを無効化（テキスト選択を可能に）
        if app.show_preview && mouse_captured {
            execute!(io::stdout(), DisableMouseCapture)?;
            mouse_captured = false;
        } else if !app.show_preview && !mouse_captured {
            execute!(io::stdout(), EnableMouseCapture)?;
            mouse_captured = true;
        }

        terminal.draw(|frame| ui::render(frame, &mut app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    let visible_lines = app.preview_visible_lines;
                    if input::handle_key(&mut app, key.code, visible_lines) {
                        break;
                    }
                }
            }
            Event::Mouse(mouse) => {
                // マウスキャプチャ中のみマウスイベントを処理
                if mouse_captured {
                    let visible_lines = app.preview_visible_lines;
                    input::handle_mouse(&mut app, mouse, visible_lines);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
