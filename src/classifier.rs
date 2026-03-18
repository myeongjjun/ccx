use crate::models::{RawSessionMetadata, ScoreHints, SessionRecord, SessionType};

const CC_MARKERS: &[&str] = &["claude", "claude-code", "claude_code", "cc"];
const CX_MARKERS: &[&str] = &["codex", "cx"];
const REPO_STOP_NAMES: &[&str] = &[
    "",
    "/",
    "users",
    "home",
    "work",
    "workspace",
    "projects",
    "src",
    "repos",
];
const REPO_ROOT_MARKERS: &[&str] = &[
    "work",
    "workspace",
    "projects",
    "src",
    "repos",
    "code",
    "dev",
];

pub fn normalize_session(raw: RawSessionMetadata) -> SessionRecord {
    let normalized_badge = normalize_optional_field(raw.badge.clone());
    let normalized_cwd = normalize_optional_field(raw.cwd.clone());
    let normalized_foreground_command = normalize_optional_field(raw.foreground_command.clone());
    let normalized_process_name = normalize_optional_field(raw.process_name.clone());
    let normalized_raw = RawSessionMetadata {
        badge: normalized_badge.clone(),
        cwd: normalized_cwd.clone(),
        foreground_command: normalized_foreground_command.clone(),
        process_name: normalized_process_name.clone(),
        ..raw.clone()
    };
    let cwd = normalized_cwd.or_else(|| infer_cwd_from_title(raw.title.as_deref()));
    let foreground_command = normalized_foreground_command
        .or(normalized_process_name)
        .or_else(|| infer_command_from_title(raw.title.as_deref()));
    let session_type = infer_session_type(&normalized_raw);
    let repo_name = extract_repo_name(cwd.as_deref());
    let score_hints = build_score_hints(
        raw.title.as_deref(),
        normalized_badge.as_deref(),
        cwd.as_deref(),
        raw.last_active_at,
        &session_type,
        repo_name.as_deref(),
    );
    let title = raw
        .title
        .clone()
        .unwrap_or_else(|| default_title(&session_type, repo_name.as_deref(), cwd.as_deref()));

    SessionRecord {
        session_id: raw.session_id,
        window_id: raw.window_id,
        tab_id: raw.tab_id,
        miniaturized: raw.miniaturized,
        title,
        badge: normalized_badge,
        cwd,
        repo_name,
        foreground_command,
        session_type,
        last_active_at: raw.last_active_at,
        score_hints,
    }
}

pub fn infer_session_type(raw: &RawSessionMetadata) -> SessionType {
    let haystacks = [
        raw.foreground_command.as_deref().unwrap_or_default(),
        raw.process_name.as_deref().unwrap_or_default(),
        raw.title.as_deref().unwrap_or_default(),
        raw.badge.as_deref().unwrap_or_default(),
    ]
    .join(" ");

    let tokens = split_keywords(&haystacks);
    if contains_marker(&tokens, CC_MARKERS) {
        SessionType::Cc
    } else if contains_marker(&tokens, CX_MARKERS) {
        SessionType::Cx
    } else {
        SessionType::Other
    }
}

pub fn extract_repo_name(cwd: Option<&str>) -> Option<String> {
    let cwd = cwd?;
    let candidates: Vec<String> = cwd
        .split('/')
        .filter(|part| !part.is_empty())
        .map(normalize_token)
        .collect();

    if cwd.starts_with("~/") {
        return candidates.last().cloned();
    }

    for (index, part) in candidates.iter().enumerate() {
        if REPO_ROOT_MARKERS.contains(&part.as_str()) && index + 1 < candidates.len() {
            return Some(candidates[index + 1].clone());
        }
    }

    candidates
        .into_iter()
        .find(|part| !REPO_STOP_NAMES.contains(&part.as_str()))
}

pub fn build_score_hints(
    title: Option<&str>,
    badge: Option<&str>,
    cwd: Option<&str>,
    last_active_at: Option<i64>,
    session_type: &SessionType,
    repo_name: Option<&str>,
) -> ScoreHints {
    let title = normalize_token(title.unwrap_or_default());
    let title_tokens = split_keywords(&title);
    let badge = normalize_token(badge.unwrap_or_default());
    let badge_tokens = split_keywords(&badge);
    let exact_type = title_tokens
        .first()
        .map(|token| token == session_type_value(session_type))
        .unwrap_or(false)
        || badge_tokens
            .iter()
            .any(|token| token == session_type_value(session_type));
    let exact_repo = repo_name
        .map(|repo| split_keywords(&title).iter().any(|token| token == repo))
        .unwrap_or(false);
    let exact_path_segment = match (repo_name, cwd) {
        (Some(repo), Some(cwd)) => normalize_token(cwd).contains(&format!("/{repo}/")),
        _ => false,
    };
    let recently_active = last_active_at.is_some();

    ScoreHints {
        exact_type,
        exact_repo,
        exact_path_segment,
        recently_active,
    }
}

fn default_title(session_type: &SessionType, repo_name: Option<&str>, cwd: Option<&str>) -> String {
    let kind = session_type.as_label();
    if let Some(repo_name) = repo_name {
        format!("{kind} | {repo_name}")
    } else if let Some(cwd) = cwd {
        format!("{kind} | {cwd}")
    } else {
        kind.to_string()
    }
}

fn session_type_value(session_type: &SessionType) -> &'static str {
    match session_type {
        SessionType::Cc => "cc",
        SessionType::Cx => "cx",
        SessionType::Other => "other",
    }
}

fn contains_marker(tokens: &[String], markers: &[&str]) -> bool {
    tokens.iter().any(|token| markers.contains(&token.as_str()))
}

fn normalize_token(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_optional_field(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn infer_cwd_from_title(title: Option<&str>) -> Option<String> {
    let suffix = title_suffix(title)?;
    let candidate = suffix.trim();
    if candidate.starts_with("~/") || candidate.starts_with('/') {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn infer_command_from_title(title: Option<&str>) -> Option<String> {
    title_prefix(title).map(|value| value.trim().to_string())
}

fn title_prefix(title: Option<&str>) -> Option<&str> {
    let title = title?;
    title
        .rsplit_once('—')
        .map(|(prefix, _)| prefix.trim())
        .filter(|value| !value.is_empty())
}

fn title_suffix(title: Option<&str>) -> Option<&str> {
    let title = title?;
    title
        .rsplit_once('—')
        .map(|(_, suffix)| suffix.trim())
        .filter(|value| !value.is_empty())
}

fn split_keywords(value: &str) -> Vec<String> {
    value
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | '_' | '-' | '|' | ':' | '/'))
        .filter(|part| !part.trim().is_empty())
        .map(normalize_token)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{extract_repo_name, infer_session_type, normalize_session};
    use crate::models::{RawSessionMetadata, SessionType};

    #[test]
    fn infers_claude_session_type() {
        let raw = RawSessionMetadata {
            session_id: "1".into(),
            window_id: "w1".into(),
            tab_id: "t1".into(),
            title: Some("CC | auth | bugfix".into()),
            foreground_command: Some("claude".into()),
            ..RawSessionMetadata::default()
        };

        assert_eq!(infer_session_type(&raw), SessionType::Cc);
    }

    #[test]
    fn extracts_repo_name_after_workspace_root() {
        assert_eq!(
            extract_repo_name(Some("/Users/me/workspace/repo-a/feature/auth")),
            Some("repo-a".into())
        );
    }

    #[test]
    fn extracts_repo_name_from_home_shorthand_path() {
        assert_eq!(
            extract_repo_name(Some("~/personal/ccx")),
            Some("ccx".into())
        );
    }

    #[test]
    fn normalizes_session_with_hints() {
        let raw = RawSessionMetadata {
            session_id: "1".into(),
            window_id: "w1".into(),
            tab_id: "t1".into(),
            title: Some("CX | repo-b | api tests".into()),
            badge: Some("cx".into()),
            cwd: Some("/Users/me/work/repo-b/tests/api".into()),
            foreground_command: Some("codex".into()),
            last_active_at: Some(123),
            ..RawSessionMetadata::default()
        };

        let session = normalize_session(raw);

        assert_eq!(session.session_type, SessionType::Cx);
        assert_eq!(session.repo_name.as_deref(), Some("repo-b"));
        assert!(session.score_hints.exact_type);
        assert!(session.score_hints.recently_active);
    }

    #[test]
    fn exact_type_requires_token_boundary() {
        let raw = RawSessionMetadata {
            session_id: "1".into(),
            window_id: "w1".into(),
            tab_id: "t1".into(),
            title: Some("ccache rebuild".into()),
            foreground_command: Some("zsh".into()),
            ..RawSessionMetadata::default()
        };

        let session = normalize_session(raw);

        assert!(!session.score_hints.exact_type);
    }

    #[test]
    fn infers_cwd_and_command_from_title_when_missing() {
        let raw = RawSessionMetadata {
            session_id: "1".into(),
            window_id: "w1".into(),
            tab_id: "t1".into(),
            title: Some("codex — ~/personal/ccx".into()),
            ..RawSessionMetadata::default()
        };

        let session = normalize_session(raw);

        assert_eq!(session.cwd.as_deref(), Some("~/personal/ccx"));
        assert_eq!(session.repo_name.as_deref(), Some("ccx"));
        assert_eq!(session.foreground_command.as_deref(), Some("codex"));
    }

    #[test]
    fn empty_foreground_command_still_allows_title_fallback() {
        let raw = RawSessionMetadata {
            session_id: "1".into(),
            window_id: "w1".into(),
            tab_id: "t1".into(),
            title: Some("-zsh — ~/personal/ccx".into()),
            foreground_command: Some(String::new()),
            ..RawSessionMetadata::default()
        };

        let session = normalize_session(raw);

        assert_eq!(session.cwd.as_deref(), Some("~/personal/ccx"));
        assert_eq!(session.foreground_command.as_deref(), Some("-zsh"));
    }
}
