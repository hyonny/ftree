use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub depth: usize,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
}

impl TreeNode {
    pub fn new(path: PathBuf, depth: usize, parent: Option<usize>) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let is_dir = path.is_dir();
        let is_hidden = name.starts_with('.') && name != ".";

        let (size, modified) = path
            .metadata()
            .map(|m| {
                let size = if is_dir { None } else { Some(m.len()) };
                let modified = m.modified().ok();
                (size, modified)
            })
            .unwrap_or((None, None));

        Self {
            path,
            name,
            is_dir,
            is_hidden,
            depth,
            size,
            modified,
            children: Vec::new(),
            parent,
        }
    }

    /// ファイルサイズを人間が読みやすい形式で返す
    pub fn size_display(&self) -> String {
        match self.size {
            Some(size) => format_size(size),
            None => String::new(),
        }
    }

    /// 更新日時を表示用にフォーマット
    pub fn modified_display(&self) -> String {
        match self.modified {
            Some(time) => format_time(time),
            None => String::new(),
        }
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

fn format_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            // 簡易的なフォーマット: YYYY-MM-DD HH:MM
            let days = secs / 86400;
            let years = 1970 + days / 365;
            let remaining_days = days % 365;
            let months = remaining_days / 30 + 1;
            let day = remaining_days % 30 + 1;
            let hours = (secs % 86400) / 3600;
            let minutes = (secs % 3600) / 60;
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                years, months, day, hours, minutes
            )
        }
        Err(_) => String::new(),
    }
}

#[derive(Debug)]
pub struct FileTree {
    pub nodes: Vec<TreeNode>,
    pub expanded: HashMap<usize, bool>,
    pub root_path: PathBuf,
}

impl FileTree {
    pub fn new(root: &Path) -> io::Result<Self> {
        let root_path = root.canonicalize()?;
        let mut tree = Self {
            nodes: Vec::new(),
            expanded: HashMap::new(),
            root_path: root_path.clone(),
        };

        tree.build_tree(&root_path)?;

        if !tree.nodes.is_empty() {
            tree.expanded.insert(0, true);
        }

        Ok(tree)
    }

    fn build_tree(&mut self, root: &Path) -> io::Result<()> {
        let mut path_to_index: HashMap<PathBuf, usize> = HashMap::new();

        for entry in WalkDir::new(root)
            .min_depth(0)
            .max_depth(10)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            let depth = entry.depth();

            let parent_idx = if depth == 0 {
                None
            } else {
                path.parent().and_then(|p| path_to_index.get(p).copied())
            };

            let node = TreeNode::new(path.clone(), depth, parent_idx);
            let idx = self.nodes.len();
            self.nodes.push(node);
            path_to_index.insert(path, idx);

            if let Some(parent_idx) = parent_idx {
                self.nodes[parent_idx].children.push(idx);
            }
        }

        Ok(())
    }

    pub fn is_expanded(&self, idx: usize) -> bool {
        self.expanded.get(&idx).copied().unwrap_or(false)
    }

    pub fn toggle_expanded(&mut self, idx: usize) {
        if self.nodes.get(idx).is_some_and(|n| n.is_dir) {
            let current = self.is_expanded(idx);
            self.expanded.insert(idx, !current);
        }
    }

    pub fn visible_nodes(&self, show_hidden: bool) -> Vec<usize> {
        let mut visible = Vec::new();
        if self.nodes.is_empty() {
            return visible;
        }

        self.collect_visible(0, show_hidden, &mut visible);
        visible
    }

    fn collect_visible(&self, idx: usize, show_hidden: bool, visible: &mut Vec<usize>) {
        let node = &self.nodes[idx];

        // 隠しファイルをスキップ（ルートノードは除く）
        if !show_hidden && node.is_hidden && node.depth > 0 {
            return;
        }

        visible.push(idx);

        if node.is_dir && self.is_expanded(idx) {
            for &child_idx in &node.children {
                self.collect_visible(child_idx, show_hidden, visible);
            }
        }
    }

    pub fn get_path(&self, idx: usize) -> Option<&Path> {
        self.nodes.get(idx).map(|n| n.path.as_path())
    }

    /// ルートからの相対パスを取得
    pub fn get_relative_path(&self, idx: usize) -> Option<PathBuf> {
        self.nodes.get(idx).map(|n| {
            n.path
                .strip_prefix(&self.root_path)
                .map(|p| {
                    if p.as_os_str().is_empty() {
                        PathBuf::from(".")
                    } else {
                        p.to_path_buf()
                    }
                })
                .unwrap_or_else(|_| n.path.clone())
        })
    }
}

