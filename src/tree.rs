use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
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

        Self {
            path,
            name,
            is_dir,
            depth,
            children: Vec::new(),
            parent,
        }
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
            .filter_entry(|e| !is_hidden(e))
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

    pub fn visible_nodes(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        if self.nodes.is_empty() {
            return visible;
        }

        self.collect_visible(0, &mut visible);
        visible
    }

    fn collect_visible(&self, idx: usize, visible: &mut Vec<usize>) {
        visible.push(idx);

        let node = &self.nodes[idx];
        if node.is_dir && self.is_expanded(idx) {
            for &child_idx in &node.children {
                self.collect_visible(child_idx, visible);
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

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.') && s != ".")
}
