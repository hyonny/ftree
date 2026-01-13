use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::git::GitStatus;

pub fn render(frame: &mut Frame, app: &mut App) {
    let areas = if app.mode == Mode::Search {
        let [main_area, search_area, status_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(frame.area());
        render_search_input(frame, app, search_area);
        (main_area, status_area)
    } else {
        let [main_area, status_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
        (main_area, status_area)
    };

    render_tree(frame, app, areas.0);
    render_status(frame, app, areas.1);

    if app.show_help {
        render_help(frame);
    }

    if app.show_preview {
        render_preview(frame, app);
    }
}

fn render_search_input(frame: &mut Frame, app: &App, area: Rect) {
    let match_count = app.search.match_count();
    let title = if match_count > 0 {
        format!(
            " Search [{}/{}] (↑↓:move Enter:jump Esc:cancel) ",
            app.search.current_index + 1,
            match_count
        )
    } else if app.search.query.is_empty() {
        " Search ".to_string()
    } else {
        " Search [no matches] ".to_string()
    };

    let input = Paragraph::new(app.search.query.as_str())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(input, area);
}

fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    // マウス操作用にツリーエリアの位置を保存（ボーダー分を考慮）
    app.tree_area_y = area.y + 1;
    app.tree_area_height = area.height.saturating_sub(2);

    let visible = app.visible_nodes();
    let is_searching = app.mode == Mode::Search && !app.search.query.is_empty();

    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let node = &app.tree.nodes[idx];
            let indent = "  ".repeat(node.depth);

            let icon = if node.is_dir {
                if app.tree.is_expanded(idx) {
                    "\u{f07c} "
                } else {
                    "\u{f07b} "
                }
            } else {
                "\u{f15b} "
            };

            // Gitステータスに基づいてスタイルを調整
            let git_status = app.get_git_status(idx);
            let base_style = match git_status {
                Some(GitStatus::Modified) => Style::default().fg(Color::Yellow),
                Some(GitStatus::Added) => Style::default().fg(Color::Green),
                Some(GitStatus::Deleted) => Style::default().fg(Color::Red),
                Some(GitStatus::Untracked) => Style::default().fg(Color::Gray),
                Some(GitStatus::Conflicted) => Style::default().fg(Color::Magenta),
                _ if node.is_dir => Style::default().fg(Color::Blue),
                _ => Style::default(),
            };

            // 検索中でマッチしている場合はハイライト
            let (name_spans, is_current) = if is_searching {
                let is_current = app.search.is_current_match(idx);
                if let Some(search_match) = app.search.get_match(idx) {
                    (highlight_matches(&node.name, &search_match.indices, base_style), is_current)
                } else {
                    (vec![Span::styled(&node.name, base_style.fg(Color::DarkGray))], false)
                }
            } else {
                (vec![Span::styled(&node.name, base_style)], false)
            };

            let mut spans = vec![Span::raw(indent), Span::styled(icon, base_style)];

            // 現在のマッチには名前の後にマーカーを追加
            spans.extend(name_spans);
            if is_current {
                spans.push(Span::styled(" <", Style::default().fg(Color::Green)));
            }

            // Gitステータスマーカー
            if let Some(status) = git_status {
                let marker_style = match status {
                    GitStatus::Modified => Style::default().fg(Color::Yellow),
                    GitStatus::Added => Style::default().fg(Color::Green),
                    GitStatus::Deleted => Style::default().fg(Color::Red),
                    GitStatus::Untracked => Style::default().fg(Color::Gray),
                    GitStatus::Conflicted => Style::default().fg(Color::Magenta),
                    _ => Style::default(),
                };
                spans.push(Span::styled(format!(" [{}]", status.marker()), marker_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = app.tree.root_path.to_string_lossy().to_string();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.cursor));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(ref msg) = app.message {
        msg.clone()
    } else {
        let visible = app.visible_nodes();
        let total = visible.len();
        let current = app.cursor + 1;

        // 選択中のファイル情報を取得
        let file_info = app.selected_index().map(|idx| {
            let node = &app.tree.nodes[idx];
            let size = node.size_display();
            let modified = node.modified_display();
            if size.is_empty() && modified.is_empty() {
                String::new()
            } else if size.is_empty() {
                format!(" | {}", modified)
            } else {
                format!(" | {} {}", size, modified)
            }
        }).unwrap_or_default();

        format!(
            " {}/{}{}  ?:help",
            current, total, file_info
        )
    };

    let status = Paragraph::new(status_text).style(Style::default().fg(Color::White));
    frame.render_widget(status, area);
}

fn render_help(frame: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  j / ↓      ", Style::default().fg(Color::Yellow)),
            Span::raw("下に移動"),
        ]),
        Line::from(vec![
            Span::styled("  k / ↑      ", Style::default().fg(Color::Yellow)),
            Span::raw("上に移動"),
        ]),
        Line::from(vec![
            Span::styled("  Enter/l/→  ", Style::default().fg(Color::Yellow)),
            Span::raw("ディレクトリを開く"),
        ]),
        Line::from(vec![
            Span::styled("  ←          ", Style::default().fg(Color::Yellow)),
            Span::raw("閉じる/親を閉じる"),
        ]),
        Line::from(vec![
            Span::styled("  Space      ", Style::default().fg(Color::Yellow)),
            Span::raw("開閉トグル"),
        ]),
        Line::from(vec![
            Span::styled("  h / BS     ", Style::default().fg(Color::Yellow)),
            Span::raw("親ディレクトリへ移動"),
        ]),
        Line::from(vec![
            Span::styled("  y          ", Style::default().fg(Color::Yellow)),
            Span::raw("パスをコピー"),
        ]),
        Line::from(vec![
            Span::styled("  Y          ", Style::default().fg(Color::Yellow)),
            Span::raw("絶対パスをコピー"),
        ]),
        Line::from(vec![
            Span::styled("  .          ", Style::default().fg(Color::Yellow)),
            Span::raw("隠しファイル表示切替"),
        ]),
        Line::from(vec![
            Span::styled("  /          ", Style::default().fg(Color::Yellow)),
            Span::raw("検索 (↑↓:次/前)"),
        ]),
        Line::from(vec![
            Span::styled("  R          ", Style::default().fg(Color::Yellow)),
            Span::raw("Gitステータス更新"),
        ]),
        Line::from(vec![
            Span::styled("  p          ", Style::default().fg(Color::Yellow)),
            Span::raw("ファイルプレビュー"),
        ]),
        Line::from(vec![
            Span::styled("  ? / F1     ", Style::default().fg(Color::Yellow)),
            Span::raw("ヘルプ表示/閉じる"),
        ]),
        Line::from(vec![
            Span::styled("  q          ", Style::default().fg(Color::Yellow)),
            Span::raw("終了"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Mouse: click=select, dblclick=toggle, scroll=move",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Press ? / F1 / Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    let area = centered_rect(55, 22, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn highlight_matches(text: &str, indices: &[u32], base_style: Style) -> Vec<Span<'static>> {
    let highlight_style = base_style.fg(Color::Red).add_modifier(Modifier::BOLD);
    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut current_span = String::new();
    let mut in_highlight = false;

    for (i, c) in chars.iter().enumerate() {
        let should_highlight = indices.contains(&(i as u32));

        if should_highlight != in_highlight {
            if !current_span.is_empty() {
                let style = if in_highlight {
                    highlight_style
                } else {
                    base_style
                };
                spans.push(Span::styled(current_span.clone(), style));
                current_span.clear();
            }
            in_highlight = should_highlight;
        }

        current_span.push(*c);
    }

    if !current_span.is_empty() {
        let style = if in_highlight {
            highlight_style
        } else {
            base_style
        };
        spans.push(Span::styled(current_span, style));
    }

    spans
}

fn render_preview(frame: &mut Frame, app: &mut App) {
    let Some(content) = &app.preview_content else {
        return;
    };

    // 画面の90%を使用
    let area = frame.area();
    let width = (area.width as f32 * 0.9) as u16;
    let height = (area.height as f32 * 0.9) as u16;
    let preview_area = centered_rect(width.max(40), height.max(10), area);

    // 表示可能行数を計算（ボーダーとタイトル分を引く）
    let visible_lines = preview_area.height.saturating_sub(2) as usize;
    app.preview_visible_lines = visible_lines;

    // タイトル（ファイル名とスクロール情報）
    let total_lines = content.lines.len();
    let title = if total_lines > visible_lines {
        format!(
            " {} [{}-{}/{}] (j/k:scroll, Esc/p/q:close) ",
            content.file_name,
            app.preview_scroll + 1,
            (app.preview_scroll + visible_lines).min(total_lines),
            total_lines
        )
    } else {
        format!(" {} (Esc/p/q:close) ", content.file_name)
    };

    // 表示する行を取得
    let lines: Vec<Line> = content
        .lines
        .iter()
        .skip(app.preview_scroll)
        .take(visible_lines)
        .map(|line| {
            // 行が長すぎる場合は切り詰め
            let display_width = (preview_area.width - 2) as usize;
            let truncated: String = line.chars().take(display_width).collect();
            Line::from(truncated)
        })
        .collect();

    let preview = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    frame.render_widget(Clear, preview_area);
    frame.render_widget(preview, preview_area);
}
