use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::OnceLock;

use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;

const MAX_LINES: usize = 1000;
const BINARY_CHECK_BYTES: usize = 8192;

// グローバルな構文セットとテーマセット（遅延初期化）
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

/// ハイライト付きのテキストスパン
#[derive(Debug, Clone)]
pub struct HighlightedSpan {
    pub text: String,
    pub fg: Option<(u8, u8, u8)>,
}

/// ハイライト付きの行
pub type HighlightedLine = Vec<HighlightedSpan>;

#[derive(Debug)]
pub struct PreviewContent {
    pub lines: Vec<HighlightedLine>,
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
                lines: vec![vec![HighlightedSpan {
                    text: "<Directory>".to_string(),
                    fg: None,
                }]],
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
                    lines: vec![vec![HighlightedSpan {
                        text: format!("Error: {}", e),
                        fg: Some((255, 100, 100)),
                    }]],
                    is_binary: false,
                    total_lines: 1,
                    file_name,
                };
            }
        };

        // バイナリ判定
        if is_binary_file(path) {
            return Self {
                lines: vec![vec![HighlightedSpan {
                    text: "<Binary file>".to_string(),
                    fg: Some((128, 128, 128)),
                }]],
                is_binary: true,
                total_lines: 1,
                file_name,
            };
        }

        // テキストファイルを読み込み
        let reader = BufReader::new(file);
        let mut raw_lines = Vec::new();
        let mut total_lines = 0;

        for line in reader.lines() {
            total_lines += 1;
            if raw_lines.len() < MAX_LINES {
                match line {
                    Ok(l) => raw_lines.push(l),
                    Err(_) => {
                        // UTF-8デコードエラー -> バイナリとして扱う
                        return Self {
                            lines: vec![vec![HighlightedSpan {
                                text: "<Binary file>".to_string(),
                                fg: Some((128, 128, 128)),
                            }]],
                            is_binary: true,
                            total_lines: 1,
                            file_name,
                        };
                    }
                }
            }
        }

        // シンタックスハイライトを適用
        let highlighted_lines = highlight_lines(path, &raw_lines);

        Self {
            lines: highlighted_lines,
            is_binary: false,
            total_lines,
            file_name,
        }
    }
}

/// 行にシンタックスハイライトを適用
fn highlight_lines(path: &Path, lines: &[String]) -> Vec<HighlightedLine> {
    let ss = get_syntax_set();
    let ts = get_theme_set();

    // 拡張子から構文を判定
    let syntax = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| ss.find_syntax_by_extension(ext))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    // テーマを取得（base16-ocean.dark はターミナル向けに良い）
    let theme = &ts.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    lines
        .iter()
        .map(|line| {
            // 行末に改行を追加（syntectの要件）
            let line_with_newline = format!("{}\n", line);
            match highlighter.highlight_line(&line_with_newline, ss) {
                Ok(ranges) => convert_to_highlighted_spans(ranges),
                Err(_) => vec![HighlightedSpan {
                    text: line.clone(),
                    fg: None,
                }],
            }
        })
        .collect()
}

/// syntectのスタイルをHighlightedSpanに変換
fn convert_to_highlighted_spans(ranges: Vec<(Style, &str)>) -> HighlightedLine {
    ranges
        .into_iter()
        .map(|(style, text)| {
            // 改行を除去
            let text = text.trim_end_matches('\n').to_string();
            let fg = Some((
                style.foreground.r,
                style.foreground.g,
                style.foreground.b,
            ));
            HighlightedSpan { text, fg }
        })
        .filter(|span| !span.text.is_empty())
        .collect()
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
