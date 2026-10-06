//! 所有 session 的狀態表：收 mod 的回報、移除已結束的 session、挑出最需要關注的一個。
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub session_id: String,
    pub cwd: String,
    pub state: String,
    #[serde(default)]
    pub detail: String,
    pub at: u64,
}

/// Claude Code 寫在 ~/.claude/sessions/<pid>.json 的資訊。
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub name: String,
    pub background: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub session_id: String,
    pub name: String,
    pub cwd: String,
    pub state: String,
    pub detail: String,
    pub background: bool,
    pub at: u64,
}

#[derive(Default)]
pub struct Board {
    sessions: HashMap<String, SessionView>,
}

fn urgency(state: &str) -> u8 {
    match state {
        "asking" => 5,
        "error" => 4,
        "working" => 3,
        "thinking" => 2,
        "complete" => 1,
        _ => 0,
    }
}

impl Board {
    pub fn apply(&mut self, report: Report) {
        if report.state == "ended" {
            self.sessions.remove(&report.session_id);
            return;
        }
        let entry = self.sessions.entry(report.session_id.clone()).or_insert_with(|| SessionView {
            name: report.session_id.chars().take(8).collect(),
            session_id: report.session_id.clone(),
            cwd: String::new(),
            state: String::new(),
            detail: String::new(),
            background: false,
            at: 0,
        });
        entry.cwd = report.cwd;
        entry.state = report.state;
        entry.detail = report.detail;
        entry.at = report.at;
    }

    /// 只留下 session 檔還在的 session，順便更新名稱與前景／背景。
    pub fn retain_live(&mut self, live: &[SessionInfo]) {
        self.sessions.retain(|id, _| live.iter().any(|info| &info.session_id == id));
        for info in live {
            if let Some(view) = self.sessions.get_mut(&info.session_id) {
                view.name = info.name.clone();
                view.background = info.background;
            }
        }
    }

    /// 依優先順序由高到低排；同級的話最近更新的在前，所以第一筆就是最需要關注的。
    pub fn sessions(&self) -> Vec<SessionView> {
        let mut list: Vec<SessionView> = self.sessions.values().cloned().collect();
        list.sort_by(|a, b| urgency(&b.state).cmp(&urgency(&a.state)).then(b.at.cmp(&a.at)));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(id: &str, state: &str, at: u64) -> Report {
        Report { session_id: id.into(), cwd: "D:\\x".into(), state: state.into(), detail: String::new(), at }
    }

    #[test]
    fn asking_beats_working_even_if_older() {
        let mut board = Board::default();
        board.apply(report("a", "asking", 1));
        board.apply(report("b", "working", 2));
        assert_eq!(board.sessions()[0].session_id, "a");
    }

    #[test]
    fn same_level_prefers_latest() {
        let mut board = Board::default();
        board.apply(report("a", "thinking", 1));
        board.apply(report("b", "thinking", 2));
        assert_eq!(board.sessions()[0].session_id, "b");
    }

    #[test]
    fn ended_removes_session() {
        let mut board = Board::default();
        board.apply(report("a", "working", 1));
        board.apply(report("a", "ended", 2));
        assert!(board.sessions().is_empty());
    }

    #[test]
    fn retain_live_drops_missing_sessions_and_refreshes_names() {
        let mut board = Board::default();
        board.apply(report("a", "idle", 1));
        board.apply(report("b", "idle", 1));
        let live = vec![SessionInfo { session_id: "a".into(), name: "main-session".into(), background: true }];
        board.retain_live(&live);
        let sessions = board.sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "main-session");
        assert!(sessions[0].background);
    }
}
