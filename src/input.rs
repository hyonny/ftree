use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, Mode};
use crate::clipboard;

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(300);

/// キー入力を処理する。終了する場合は true を返す。
pub fn handle_key(app: &mut App, key: KeyCode, preview_visible_lines: usize) -> bool {
    // プレビュー表示中
    if app.show_preview {
        return handle_preview_mode(app, key, preview_visible_lines);
    }

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

fn handle_preview_mode(app: &mut App, key: KeyCode, visible_lines: usize) -> bool {
    match key {
        KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('q') => {
            app.close_preview();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.preview_scroll_down(visible_lines);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.preview_scroll_up();
        }
        KeyCode::Char('n') => {
            app.toggle_preview_line_numbers();
        }
        _ => {}
    }
    false
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

        KeyCode::Char('R') => {
            app.refresh_git_status();
        }

        KeyCode::Char('p') => {
            app.show_preview();
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

/// マウス入力を処理する
pub fn handle_mouse(app: &mut App, mouse: MouseEvent, preview_visible_lines: usize) {
    // ヘルプ表示中や検索モード中はマウス操作を無視
    if app.show_help || app.mode == Mode::Search {
        return;
    }

    // プレビュー表示中はスクロールをプレビューに適用
    if app.show_preview {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                app.preview_scroll_up();
            }
            MouseEventKind::ScrollDown => {
                app.preview_scroll_down(preview_visible_lines);
            }
            _ => {}
        }
        return;
    }

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let now = Instant::now();
            let is_double_click = app
                .last_click
                .map(|(time, row, col)| {
                    now.duration_since(time) < DOUBLE_CLICK_THRESHOLD
                        && row == mouse.row
                        && col == mouse.column
                })
                .unwrap_or(false);

            if let Some(row) = calculate_tree_row(app, mouse.row) {
                let visible_count = app.visible_nodes().len();
                if row < visible_count {
                    app.cursor = row;
                    if is_double_click {
                        // ダブルクリックでディレクトリを開閉
                        app.toggle_selected();
                        app.last_click = None;
                    } else {
                        app.last_click = Some((now, mouse.row, mouse.column));
                    }
                }
            } else {
                app.last_click = Some((now, mouse.row, mouse.column));
            }
        }
        MouseEventKind::ScrollUp => {
            // スクロールアップで上に移動
            app.move_up();
        }
        MouseEventKind::ScrollDown => {
            // スクロールダウンで下に移動
            app.move_down();
        }
        _ => {}
    }
}

/// マウスのY座標からツリー内の行番号を計算
fn calculate_tree_row(app: &App, mouse_y: u16) -> Option<usize> {
    if mouse_y >= app.tree_area_y && mouse_y < app.tree_area_y + app.tree_area_height {
        Some((mouse_y - app.tree_area_y) as usize)
    } else {
        None
    }
}
