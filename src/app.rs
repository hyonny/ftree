use std::env;
use std::io;

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
    pub search_query: String,
    pub show_hidden: bool,
    pub show_help: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let cwd = env::current_dir()?;
        let tree = FileTree::new(&cwd)?;

        Ok(Self {
            tree,
            cursor: 0,
            mode: Mode::Normal,
            search_query: String::new(),
            show_hidden: false,
            show_help: false,
            message: None,
        })
    }

    pub fn visible_nodes(&self) -> Vec<usize> {
        self.tree.visible_nodes()
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
}
