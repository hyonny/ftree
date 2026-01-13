# ftree

ターミナル上で動作するファイルツリー TUI アプリケーション。

Rust + [ratatui](https://ratatui.rs/) で構築。

## 機能

- ファイルツリー表示（カレントディレクトリ以下）
- キーボード & マウス操作
- ファジー検索（nucleo-matcher）
- Git ステータス表示
- ファイルプレビュー（シンタックスハイライト付き）
- パスのクリップボードコピー

## インストール

```bash
# リポジトリをクローン
git clone https://github.com/hyonny/ftree.git
cd ftree

# インストール
cargo install --path .
```

## 使い方

```bash
cd /path/to/directory
ftree
```

## キーバインド

### 基本操作

| キー | 動作 |
|------|------|
| `j` / `↓` | 下に移動 |
| `k` / `↑` | 上に移動 |
| `Enter` / `l` / `→` | ディレクトリを開く |
| `←` | 閉じる / 親を閉じる |
| `Space` | 開閉トグル |
| `h` / `Backspace` | 親ディレクトリへ移動 |
| `q` | 終了 |

### ファイル操作

| キー | 動作 |
|------|------|
| `y` | 相対パスをコピー |
| `Y` | 絶対パスをコピー |
| `p` | ファイルプレビュー |

### 検索・表示

| キー | 動作 |
|------|------|
| `/` | 検索モード |
| `.` | 隠しファイル表示切替 |
| `R` | Git ステータス更新 |
| `?` / `F1` | ヘルプ表示 |

### 検索モード

| キー | 動作 |
|------|------|
| `↑` / `↓` | 前/次のマッチに移動 |
| `Enter` | 選択してジャンプ |
| `Esc` | キャンセル |

### プレビューモード

| キー | 動作 |
|------|------|
| `j` / `k` | スクロール |
| `n` | 行番号表示切替 |
| `Esc` / `p` / `q` | 閉じる |

プレビュー中はマウスでテキスト選択が可能（コピー用）。

### マウス操作

| 操作 | 動作 |
|------|------|
| クリック | 選択 |
| ダブルクリック | ディレクトリ開閉 |
| スクロール | 上下移動 |

## 依存クレート

- [ratatui](https://crates.io/crates/ratatui) - TUI フレームワーク
- [crossterm](https://crates.io/crates/crossterm) - ターミナル操作
- [walkdir](https://crates.io/crates/walkdir) - ディレクトリ走査
- [nucleo-matcher](https://crates.io/crates/nucleo-matcher) - ファジー検索
- [syntect](https://crates.io/crates/syntect) - シンタックスハイライト
- [arboard](https://crates.io/crates/arboard) - クリップボード

## ライセンス

MIT
