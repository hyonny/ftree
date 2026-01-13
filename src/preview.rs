use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

const MAX_LINES: usize = 1000;
const BINARY_CHECK_BYTES: usize = 8192;

#[derive(Debug)]
pub struct PreviewContent {
    pub lines: Vec<String>,
    pub is_binary: bool,
    pub total_lines: usize,
    pub file_name: String,
}

impl PreviewContent {
    pub fn load(path: &Path) -> Self {
        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // ディレクトリの場合
        if path.is_dir() {
            return Self {
                lines: vec!["<Directory>".to_string()],
                is_binary: false,
                total_lines: 1,
                file_name,
            };
        }

        // ファイルを開く
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Self {
                    lines: vec![format!("Error: {}", e)],
                    is_binary: false,
                    total_lines: 1,
                    file_name,
                };
            }
        };

        // バイナリ判定
        if is_binary_file(path) {
            return Self {
                lines: vec!["<Binary file>".to_string()],
                is_binary: true,
                total_lines: 1,
                file_name,
            };
        }

        // テキストファイルを読み込み
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut total_lines = 0;

        for line in reader.lines() {
            total_lines += 1;
            if lines.len() < MAX_LINES {
                match line {
                    Ok(l) => lines.push(l),
                    Err(_) => {
                        // UTF-8デコードエラー -> バイナリとして扱う
                        return Self {
                            lines: vec!["<Binary file>".to_string()],
                            is_binary: true,
                            total_lines: 1,
                            file_name,
                        };
                    }
                }
            }
        }

        Self {
            lines,
            is_binary: false,
            total_lines,
            file_name,
        }
    }
}

/// ファイルがバイナリかどうかを判定
fn is_binary_file(path: &Path) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; BINARY_CHECK_BYTES];

    let bytes_read = match reader.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    // NULLバイトが含まれていればバイナリ
    buffer[..bytes_read].contains(&0)
}
