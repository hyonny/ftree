use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let [main_area, status_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

    render_tree(frame, app, main_area);
    render_status(frame, app, status_area);

    if app.show_help {
        render_help(frame);
    }
}

fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let visible = app.visible_nodes();

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

            let style = if node.is_dir {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(icon, style),
                Span::styled(&node.name, style),
            ]);

            ListItem::new(line)
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
        format!(
            " {}/{} | q:quit j/k:move Enter/Space:toggle y:copy Y:copy-abs h:parent ?/F1:help",
            current, total
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
            Span::styled("  ? / F1     ", Style::default().fg(Color::Yellow)),
            Span::raw("ヘルプ表示/閉じる"),
        ]),
        Line::from(vec![
            Span::styled("  q          ", Style::default().fg(Color::Yellow)),
            Span::raw("終了"),
        ]),
        Line::from(""),
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

    let area = centered_rect(40, 17, frame.area());
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
