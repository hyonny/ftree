use crossterm::event::KeyCode;

use crate::app::App;
use crate::clipboard;

/// キー入力を処理する。終了する場合は true を返す。
pub fn handle_key(app: &mut App, key: KeyCode) -> bool {
    app.clear_message();

    // ヘルプ表示中は Esc か ? か F1 で閉じる
    if app.show_help {
        match key {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) => {
                app.show_help = false;
            }
            KeyCode::Char('q') => return true,
            _ => {}
        }
        return false;
    }

    match key {
        KeyCode::Char('q') => return true,

        // ヘルプ表示（? または F1）
        KeyCode::Char('?') | KeyCode::F(1) => {
            app.toggle_help();
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

        _ => {}
    }

    false
}
