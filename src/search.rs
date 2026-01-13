use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

#[derive(Debug)]
pub struct SearchState {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub current_index: usize,
    matcher: Matcher,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub node_idx: usize,
    pub score: u32,
    pub indices: Vec<u32>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_index: 0,
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.current_index = 0;
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
    }

    /// ファイル名リストに対して検索を実行
    pub fn search(&mut self, names: &[(usize, &str)]) {
        self.matches.clear();

        if self.query.is_empty() {
            return;
        }

        let pattern = Pattern::parse(&self.query, CaseMatching::Ignore, Normalization::Smart);
        let mut buf = Vec::new();

        for &(idx, name) in names {
            let haystack = Utf32Str::new(name, &mut buf);
            let mut indices = Vec::new();

            if let Some(score) = pattern.indices(haystack, &mut self.matcher, &mut indices) {
                self.matches.push(SearchMatch {
                    node_idx: idx,
                    score,
                    indices,
                });
            }
        }

        // スコア降順でソート
        self.matches.sort_by(|a, b| b.score.cmp(&a.score));

        // current_index をリセット
        self.current_index = 0;
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// 現在のマッチのノードインデックスを取得
    pub fn current_match(&self) -> Option<usize> {
        self.matches.get(self.current_index).map(|m| m.node_idx)
    }

    /// 次のマッチに移動
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_index = (self.current_index + 1) % self.matches.len();
        }
    }

    /// 前のマッチに移動
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.matches.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }

    /// 指定ノードがマッチしているか、マッチしている場合はハイライト位置も返す
    pub fn get_match(&self, node_idx: usize) -> Option<&SearchMatch> {
        self.matches.iter().find(|m| m.node_idx == node_idx)
    }

    /// 指定ノードが現在選択中のマッチかどうか
    pub fn is_current_match(&self, node_idx: usize) -> bool {
        self.current_match() == Some(node_idx)
    }
}
