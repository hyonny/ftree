use crossterm::event::KeyCode;

use crate::app::{App, Mode};
use crate::clipboard;

/// キー入力を処理する。終了する場合は true を返す。
pub fn handle_key(app: &mut App, key: KeyCode) -> bool {
    // ヘルプ表示中
    if app.show_help {
        return handle_help_mode(app, key);
    }

    // モードに応じて処理を分岐
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Search => handle_search_mode(app, key),
    }
}

fn handle_help_mode(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) => {
            app.show_help = false;
        }
        KeyCode::Char('q') => return true,
        _ => {}
    }
    false
}

fn handle_normal_mode(app: &mut App, key: KeyCode) -> bool {
    app.clear_message();

    match key {
        KeyCode::Char('q') => return true,

        KeyCode::Char('?') | KeyCode::F(1) => {
            app.toggle_help();
        }

        KeyCode::Char('/') => {
            app.enter_search_mode();
        }

        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
        }

        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
        }

        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            app.enter_directory();
        }

        KeyCode::Char(' ') => {
            app.toggle_selected();
        }

        KeyCode::Left => {
            app.collapse_or_parent();
        }

        KeyCode::Char('h') | KeyCode::Backspace => {
            app.go_to_parent();
        }

        KeyCode::Char('y') => {
            if let Some(idx) = app.selected_index() {
                if let Some(path) = app.tree.get_relative_path(idx) {
                    let path_str = path.to_string_lossy().to_string();
                    match clipboard::copy_to_clipboard(&path_str) {
                        Ok(_) => app.set_message(format!("Copied: {}", path_str)),
                        Err(e) => app.set_message(format!("Copy failed: {}", e)),
                    }
                }
            }
        }

        KeyCode::Char('Y') => {
            if let Some(idx) = app.selected_index() {
                if let Some(path) = app.tree.get_path(idx) {
                    let path_str = path.to_string_lossy().to_string();
                    match clipboard::copy_to_clipboard(&path_str) {
                        Ok(_) => app.set_message(format!("Copied: {}", path_str)),
                        Err(e) => app.set_message(format!("Copy failed: {}", e)),
                    }
                }
            }
        }

        KeyCode::Char('.') => {
            app.toggle_hidden();
            let status = if app.show_hidden { "ON" } else { "OFF" };
            app.set_message(format!("Hidden files: {}", status));
        }

        _ => {}
    }

    false
}

fn handle_search_mode(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Esc => {
            app.exit_search_mode();
        }

        KeyCode::Enter => {
            app.confirm_search();
        }

        KeyCode::Backspace => {
            app.search.pop_char();
            app.update_search();
        }

        // 次のマッチに移動
        KeyCode::Down | KeyCode::Tab => {
            app.next_search_match();
        }

        // 前のマッチに移動
        KeyCode::Up | KeyCode::BackTab => {
            app.prev_search_match();
        }

        KeyCode::Char(c) => {
            app.search.push_char(c);
            app.update_search();
        }

        _ => {}
    }

    false
}
