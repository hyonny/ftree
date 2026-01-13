use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
    Conflicted,
}

impl GitStatus {
    pub fn marker(&self) -> &'static str {
        match self {
            GitStatus::Modified => "M",
            GitStatus::Added => "A",
            GitStatus::Deleted => "D",
            GitStatus::Renamed => "R",
            GitStatus::Untracked => "?",
            GitStatus::Ignored => "!",
            GitStatus::Conflicted => "C",
        }
    }
}

/// Gitリポジトリのステータスを取得
pub fn get_git_status(root_path: &Path) -> HashMap<String, GitStatus> {
    let mut statuses = HashMap::new();

    // git status --porcelain を実行
    let output = Command::new("git")
        .args(["status", "--porcelain", "-uall"])
        .current_dir(root_path)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return statuses, // Gitリポジトリでない場合は空を返す
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }

        let index_status = line.chars().next().unwrap_or(' ');
        let worktree_status = line.chars().nth(1).unwrap_or(' ');
        let path = &line[3..];

        // リネームの場合は " -> " で分割
        let file_path = if path.contains(" -> ") {
            path.split(" -> ").last().unwrap_or(path)
        } else {
            path
        };

        let status = parse_status(index_status, worktree_status);
        if let Some(s) = status {
            statuses.insert(file_path.to_string(), s);

            // 親ディレクトリにもステータスを伝播
            let mut current = Path::new(file_path);
            while let Some(parent) = current.parent() {
                if parent.as_os_str().is_empty() {
                    break;
                }
                let parent_str = parent.to_string_lossy().to_string();
                statuses.entry(parent_str).or_insert(s);
                current = parent;
            }
        }
    }

    statuses
}

fn parse_status(index: char, worktree: char) -> Option<GitStatus> {
    // コンフリクト
    if index == 'U' || worktree == 'U' {
        return Some(GitStatus::Conflicted);
    }

    // worktreeの変更を優先
    match worktree {
        'M' => return Some(GitStatus::Modified),
        'D' => return Some(GitStatus::Deleted),
        '?' => return Some(GitStatus::Untracked),
        '!' => return Some(GitStatus::Ignored),
        _ => {}
    }

    // indexの変更
    match index {
        'M' => Some(GitStatus::Modified),
        'A' => Some(GitStatus::Added),
        'D' => Some(GitStatus::Deleted),
        'R' => Some(GitStatus::Renamed),
        '?' => Some(GitStatus::Untracked),
        '!' => Some(GitStatus::Ignored),
        _ => None,
    }
}
