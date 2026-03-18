use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    Cc,
    Cx,
    Other,
}

impl SessionType {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Cc => "CC",
            Self::Cx => "CX",
            Self::Other => "OT",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreHints {
    #[serde(default)]
    pub exact_type: bool,
    #[serde(default)]
    pub exact_repo: bool,
    #[serde(default)]
    pub exact_path_segment: bool,
    #[serde(default)]
    pub recently_active: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawSessionMetadata {
    pub session_id: String,
    pub window_id: String,
    pub tab_id: String,
    #[serde(default)]
    pub miniaturized: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub badge: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub foreground_command: Option<String>,
    #[serde(default)]
    pub process_name: Option<String>,
    #[serde(default)]
    pub tty: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub window_id: String,
    pub tab_id: String,
    #[serde(default)]
    pub miniaturized: bool,
    pub title: String,
    #[serde(default)]
    pub badge: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub repo_name: Option<String>,
    #[serde(default)]
    pub foreground_command: Option<String>,
    pub session_type: SessionType,
    #[serde(default)]
    pub last_active_at: Option<i64>,
    #[serde(default)]
    pub score_hints: ScoreHints,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub session: SessionRecord,
    pub score: i64,
    pub matched_terms: Vec<String>,
    pub score_breakdown: BTreeMap<String, i64>,
}

impl SearchResult {
    pub fn render_line(&self, explain: bool) -> String {
        let session = &self.session;
        let mut line = format!(
            "{:>3}  {:<2}  {:<20}  {}  {}",
            self.score,
            session.session_type.as_label(),
            session.repo_name.as_deref().unwrap_or("-"),
            session.cwd.as_deref().unwrap_or("-"),
            session.title
        );
        if explain {
            line.push_str(&format!(
                "  matched={}  breakdown={:?}",
                if self.matched_terms.is_empty() {
                    "-".to_string()
                } else {
                    self.matched_terms.join(",")
                },
                self.score_breakdown
            ));
        }
        line
    }

    pub fn as_json(&self, explain: bool) -> Value {
        let mut payload = json!({
            "score": self.score,
            "session": self.session,
        });
        if explain {
            payload["matched_terms"] = json!(self.matched_terms);
            payload["score_breakdown"] = json!(self.score_breakdown);
        }
        payload
    }
}

pub fn sample_session() -> SessionRecord {
    SessionRecord {
        session_id: "session-1".into(),
        window_id: "window-1".into(),
        tab_id: "tab-1".into(),
        miniaturized: false,
        title: "CC | repo-a | auth fix".into(),
        badge: Some("cc".into()),
        cwd: Some("/Users/me/work/repo-a/feature/auth".into()),
        repo_name: Some("repo-a".into()),
        foreground_command: Some("claude".into()),
        session_type: SessionType::Cc,
        last_active_at: Some(1_710_000_000),
        score_hints: ScoreHints {
            exact_type: true,
            exact_repo: false,
            exact_path_segment: true,
            recently_active: true,
        },
    }
}
