use std::collections::HashMap;
use std::env;
use std::io;
use std::time::Instant;

use crate::git::{self, GitStatus};
use crate::preview::PreviewContent;
use crate::search::SearchState;
use crate::tree::FileTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
}

pub struct App {
    pub tree: FileTree,
    pub cursor: usize,
    pub mode: Mode,
    pub search: SearchState,
    pub show_hidden: bool,
    pub show_help: bool,
    pub message: Option<String>,
    /// ツリー表示エリアのY座標（マウス操作用）
    pub tree_area_y: u16,
    pub tree_area_height: u16,
    /// ダブルクリック検出用
    pub last_click: Option<(Instant, u16, u16)>,
    /// Gitステータス（相対パス -> ステータス）
    pub git_status: HashMap<String, GitStatus>,
    /// プレビュー表示中
    pub show_preview: bool,
    /// プレビュー内容
    pub preview_content: Option<PreviewContent>,
    /// プレビューのスクロール位置
    pub preview_scroll: usize,
    /// プレビューの表示可能行数（UIから設定）
    pub preview_visible_lines: usize,
    /// プレビューで行番号を表示するか
    pub preview_show_line_numbers: bool,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let cwd = env::current_dir()?;
        let tree = FileTree::new(&cwd)?;
        let git_status = git::get_git_status(&tree.root_path);

        Ok(Self {
            tree,
            cursor: 0,
            mode: Mode::Normal,
            search: SearchState::new(),
            show_hidden: false,
            show_help: false,
            message: None,
            tree_area_y: 0,
            tree_area_height: 0,
            last_click: None,
            git_status,
            show_preview: false,
            preview_content: None,
            preview_scroll: 0,
            preview_visible_lines: 20,
            preview_show_line_numbers: true,
        })
    }

    /// Gitステータスをリフレッシュ
    pub fn refresh_git_status(&mut self) {
        self.git_status = git::get_git_status(&self.tree.root_path);
        self.set_message("Git status refreshed");
    }

    /// 指定ノードのGitステータスを取得
    pub fn get_git_status(&self, idx: usize) -> Option<GitStatus> {
        self.tree.get_relative_path(idx).and_then(|path| {
            let path_str = path.to_string_lossy().to_string();
            self.git_status.get(&path_str).copied()
        })
    }

    /// プレビューを表示
    pub fn show_preview(&mut self) {
        if let Some(idx) = self.selected_index() {
            let path = &self.tree.nodes[idx].path;
            self.preview_content = Some(PreviewContent::load(path));
            self.preview_scroll = 0;
            self.show_preview = true;
        }
    }

    /// プレビューを閉じる
    pub fn close_preview(&mut self) {
        self.show_preview = false;
        self.preview_content = None;
        self.preview_scroll = 0;
    }

    /// プレビューを上にスクロール
    pub fn preview_scroll_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll -= 1;
        }
    }

    /// プレビューを下にスクロール
    pub fn preview_scroll_down(&mut self, visible_lines: usize) {
        if let Some(content) = &self.preview_content {
            let max_scroll = content.lines.len().saturating_sub(visible_lines);
            if self.preview_scroll < max_scroll {
                self.preview_scroll += 1;
            }
        }
    }

    /// プレビューの行番号表示を切替
    pub fn toggle_preview_line_numbers(&mut self) {
        self.preview_show_line_numbers = !self.preview_show_line_numbers;
    }

    pub fn visible_nodes(&self) -> Vec<usize> {
        self.tree.visible_nodes(self.show_hidden)
    }

    pub fn selected_index(&self) -> Option<usize> {
        let visible = self.visible_nodes();
        visible.get(self.cursor).copied()
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let visible_count = self.visible_nodes().len();
        if self.cursor + 1 < visible_count {
            self.cursor += 1;
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(idx) = self.selected_index() {
            self.tree.toggle_expanded(idx);
        }
    }

    pub fn enter_directory(&mut self) {
        if let Some(idx) = self.selected_index() {
            let node = &self.tree.nodes[idx];
            if node.is_dir && !self.tree.is_expanded(idx) {
                self.tree.toggle_expanded(idx);
            }
        }
    }

    pub fn go_to_parent(&mut self) {
        if let Some(idx) = self.selected_index() {
            if let Some(parent_idx) = self.tree.nodes[idx].parent {
                let visible = self.visible_nodes();
                if let Some(pos) = visible.iter().position(|&i| i == parent_idx) {
                    self.cursor = pos;
                }
            }
        }
    }

    /// 左キー: ディレクトリなら閉じる、ファイルまたは閉じたディレクトリなら親を閉じて移動
    pub fn collapse_or_parent(&mut self) {
        if let Some(idx) = self.selected_index() {
            let node = &self.tree.nodes[idx];

            // 開いているディレクトリなら閉じる
            if node.is_dir && self.tree.is_expanded(idx) {
                self.tree.toggle_expanded(idx);
            } else if let Some(parent_idx) = node.parent {
                // ファイル or 閉じたディレクトリなら、親を閉じてカーソルを移動
                if self.tree.is_expanded(parent_idx) {
                    self.tree.toggle_expanded(parent_idx);
                }
                let visible = self.visible_nodes();
                if let Some(pos) = visible.iter().position(|&i| i == parent_idx) {
                    self.cursor = pos;
                }
            }
        }
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        // カーソル位置が範囲外になった場合に調整
        let visible_count = self.visible_nodes().len();
        if self.cursor >= visible_count && visible_count > 0 {
            self.cursor = visible_count - 1;
        }
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = Mode::Search;
        self.search.clear();
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = Mode::Normal;
        self.search.clear();
    }

    pub fn confirm_search(&mut self) {
        // 現在のマッチにジャンプ
        if let Some(node_idx) = self.search.current_match() {
            self.jump_to_node(node_idx);
        }
        self.mode = Mode::Normal;
    }

    /// 次の検索結果に移動
    pub fn next_search_match(&mut self) {
        self.search.next_match();
        if let Some(node_idx) = self.search.current_match() {
            self.jump_to_node(node_idx);
        }
    }

    /// 前の検索結果に移動
    pub fn prev_search_match(&mut self) {
        self.search.prev_match();
        if let Some(node_idx) = self.search.current_match() {
            self.jump_to_node(node_idx);
        }
    }

    fn jump_to_node(&mut self, node_idx: usize) {
        let visible = self.visible_nodes();
        if let Some(pos) = visible.iter().position(|&i| i == node_idx) {
            self.cursor = pos;
        } else {
            // ノードが見えていない場合、親ディレクトリを展開
            self.expand_to_node(node_idx);
            let visible = self.visible_nodes();
            if let Some(pos) = visible.iter().position(|&i| i == node_idx) {
                self.cursor = pos;
            }
        }
    }

    fn expand_to_node(&mut self, node_idx: usize) {
        // ノードの親を辿って全て展開
        let mut current = node_idx;
        let mut parents = Vec::new();

        while let Some(parent_idx) = self.tree.nodes[current].parent {
            parents.push(parent_idx);
            current = parent_idx;
        }

        for parent_idx in parents.into_iter().rev() {
            if !self.tree.is_expanded(parent_idx) {
                self.tree.toggle_expanded(parent_idx);
            }
        }
    }

    pub fn update_search(&mut self) {
        // 全ノードの名前を収集して検索
        let names: Vec<(usize, &str)> = self
            .tree
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (i, n.name.as_str()))
            .collect();

        self.search.search(&names);
    }
}
