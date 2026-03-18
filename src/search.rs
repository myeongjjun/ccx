use std::cmp::max;
use std::collections::{BTreeMap, HashSet};

use crate::models::{SearchResult, SessionRecord};

pub const MAX_SCAN_RESULTS: usize = 200;

#[derive(Debug, Clone)]
struct IndexedSession {
    session: SessionRecord,
    tokens: Vec<String>,
    path_tokens: Vec<String>,
    repo_tokens: Vec<String>,
    title_tokens: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct QuerySpec {
    general_terms: Vec<String>,
    path_terms: Vec<String>,
    repo_terms: Vec<String>,
    title_terms: Vec<String>,
}

pub fn search_sessions(sessions: &[SessionRecord], query: &str, limit: usize) -> Vec<SearchResult> {
    let query_spec = parse_query(query);
    let has_query_terms = !query_spec.general_terms.is_empty()
        || !query_spec.path_terms.is_empty()
        || !query_spec.repo_terms.is_empty()
        || !query_spec.title_terms.is_empty();
    let mut results: Vec<_> = sessions
        .iter()
        .cloned()
        .map(build_index)
        .map(|indexed| score_indexed_session(indexed, &query_spec))
        .filter(|result| result.score > 0 && (!has_query_terms || !result.matched_terms.is_empty()))
        .collect();

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| {
                right
                    .session
                    .last_active_at
                    .cmp(&left.session.last_active_at)
            })
            .then_with(|| left.session.session_id.cmp(&right.session.session_id))
    });
    results.truncate(limit);
    results
}

fn build_index(session: SessionRecord) -> IndexedSession {
    let path_tokens = session
        .cwd
        .as_deref()
        .map(tokenize_path_segments)
        .unwrap_or_default();
    let repo_tokens = session
        .repo_name
        .as_deref()
        .map(|repo| vec![normalize_token(repo)])
        .unwrap_or_default();
    let title_tokens = split_keywords(
        &[Some(session.title.as_str()), session.badge.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
    );
    let generic_tokens = split_keywords(
        &[
            Some(session.title.as_str()),
            session.badge.as_deref(),
            session.foreground_command.as_deref(),
            Some(session_type_value(&session)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" "),
    );
    let tokens = dedupe_tokens(
        path_tokens
            .iter()
            .chain(repo_tokens.iter())
            .chain(generic_tokens.iter())
            .cloned()
            .collect(),
    );

    IndexedSession {
        session,
        tokens,
        path_tokens,
        repo_tokens,
        title_tokens,
    }
}

fn score_indexed_session(indexed: IndexedSession, query_spec: &QuerySpec) -> SearchResult {
    let terms_len = query_spec.general_terms.len()
        + query_spec.path_terms.len()
        + query_spec.repo_terms.len()
        + query_spec.title_terms.len();

    if terms_len == 0 {
        let baseline = if indexed.session.last_active_at.is_some() {
            10
        } else {
            1
        };
        let mut score_breakdown = BTreeMap::new();
        score_breakdown.insert("baseline".into(), baseline);
        return SearchResult {
            session: indexed.session,
            score: baseline,
            matched_terms: Vec::new(),
            score_breakdown,
        };
    }

    let (general_score, general_matches) = score_general_terms(&indexed, &query_spec.general_terms);
    let (path_score, path_matches) =
        score_field_terms(&query_spec.path_terms, &indexed.path_tokens, 130, 50);
    let (repo_score, repo_matches) =
        score_field_terms(&query_spec.repo_terms, &indexed.repo_tokens, 120, 45);
    let (title_score, title_matches) =
        score_field_terms(&query_spec.title_terms, &indexed.title_tokens, 110, 40);

    let mut score = general_score + path_score + repo_score + title_score;
    let mut matched_terms = Vec::new();
    matched_terms.extend(general_matches);
    matched_terms.extend(path_matches);
    matched_terms.extend(repo_matches);
    matched_terms.extend(title_matches);

    let mut score_breakdown = BTreeMap::new();
    score_breakdown.insert("general".into(), general_score);
    score_breakdown.insert("path".into(), path_score);
    score_breakdown.insert("repo".into(), repo_score);
    score_breakdown.insert("title".into(), title_score);

    if indexed.session.last_active_at.is_some() {
        score += 10;
        score_breakdown.insert("last_active".into(), 10);
    }
    if indexed.session.score_hints.recently_active {
        score += 15;
        score_breakdown.insert("hint_recently_active".into(), 15);
    }
    if indexed.session.score_hints.exact_type {
        score += 25;
        score_breakdown.insert("hint_exact_type".into(), 25);
    }

    SearchResult {
        session: indexed.session,
        score,
        matched_terms,
        score_breakdown,
    }
}

fn parse_query(query: &str) -> QuerySpec {
    let mut spec = QuerySpec::default();

    for raw_term in query.split_whitespace() {
        let normalized = normalize_token(raw_term);
        let mut parts = normalized.splitn(2, ':');
        let prefix = parts.next().unwrap_or_default();
        let value = parts.next();

        if let Some(value) = value {
            if !value.is_empty() {
                let parsed = parse_field_value(value);
                match prefix {
                    "p" | "path" => {
                        spec.path_terms.extend(parsed);
                        continue;
                    }
                    "r" | "repo" => {
                        spec.repo_terms.extend(parsed);
                        continue;
                    }
                    "t" | "title" => {
                        spec.title_terms.extend(parsed);
                        continue;
                    }
                    _ => {}
                }
            }
        }

        spec.general_terms.extend(split_keywords(&normalized));
    }

    spec.general_terms = dedupe_tokens(spec.general_terms);
    spec.path_terms = dedupe_tokens(spec.path_terms);
    spec.repo_terms = dedupe_tokens(spec.repo_terms);
    spec.title_terms = dedupe_tokens(spec.title_terms);
    spec
}

fn parse_field_value(value: &str) -> Vec<String> {
    if value.contains('/') {
        tokenize_path(value)
    } else {
        let mut tokens = split_keywords(value);
        tokens.push(normalize_token(value));
        dedupe_tokens(tokens)
    }
}

fn score_general_terms(indexed: &IndexedSession, terms: &[String]) -> (i64, Vec<String>) {
    let mut score = 0;
    let mut matches = Vec::new();

    for term in terms {
        let mut term_score = 0;
        if term == session_type_value(&indexed.session) {
            term_score = max(term_score, 120);
        }
        if indexed.path_tokens.iter().any(|token| token == term) {
            term_score = max(term_score, 100);
        }
        if indexed.repo_tokens.iter().any(|token| token == term) {
            term_score = max(term_score, 90);
        }
        if indexed.tokens.iter().any(|token| token == term) {
            term_score = max(term_score, 70);
        }

        let fuzzy = best_fuzzy_score(term, &indexed.tokens);
        if fuzzy >= 0.9 {
            term_score = max(term_score, 60);
        } else if fuzzy >= 0.75 {
            term_score = max(term_score, 35);
        } else if fuzzy >= 0.6 {
            term_score = max(term_score, 15);
        }

        if term_score > 0 {
            matches.push(term.clone());
        }
        score += term_score;
    }

    (score, matches)
}

fn score_field_terms(
    terms: &[String],
    field_tokens: &[String],
    exact_weight: i64,
    fuzzy_weight: i64,
) -> (i64, Vec<String>) {
    let mut score = 0;
    let mut matches = Vec::new();

    for term in terms {
        let mut term_score = 0;
        if field_tokens.iter().any(|token| token == term) {
            term_score = max(term_score, exact_weight);
        }

        let fuzzy = best_fuzzy_score(term, field_tokens);
        if fuzzy >= 0.9 {
            term_score = max(term_score, fuzzy_weight);
        } else if fuzzy >= 0.75 {
            term_score = max(term_score, max(fuzzy_weight - 15, 10));
        } else if fuzzy >= 0.6 {
            term_score = max(term_score, max(fuzzy_weight - 30, 5));
        }

        if term_score > 0 {
            matches.push(term.clone());
        }
        score += term_score;
    }

    (score, matches)
}

fn best_fuzzy_score(term: &str, tokens: &[String]) -> f64 {
    tokens
        .iter()
        .map(|token| similarity(term, token))
        .fold(0.0_f64, f64::max)
}

fn similarity(left: &str, right: &str) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    if left == right {
        return 1.0;
    }
    if right.contains(left) || left.contains(right) {
        return left.len().min(right.len()) as f64 / left.len().max(right.len()) as f64;
    }
    longest_common_subsequence(left, right) as f64 / left.len().max(right.len()) as f64
}

fn longest_common_subsequence(left: &str, right: &str) -> usize {
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    let mut dp = vec![vec![0; right_chars.len() + 1]; left_chars.len() + 1];

    for (i, left_char) in left_chars.iter().enumerate() {
        for (j, right_char) in right_chars.iter().enumerate() {
            dp[i + 1][j + 1] = if left_char == right_char {
                dp[i][j] + 1
            } else {
                dp[i][j + 1].max(dp[i + 1][j])
            };
        }
    }

    dp[left_chars.len()][right_chars.len()]
}

fn session_type_value(session: &SessionRecord) -> &'static str {
    match session.session_type {
        crate::models::SessionType::Cc => "cc",
        crate::models::SessionType::Cx => "cx",
        crate::models::SessionType::Other => "other",
    }
}

fn normalize_token(value: &str) -> String {
    value.trim().to_lowercase()
}

fn split_keywords(value: &str) -> Vec<String> {
    value
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | '_' | '-' | '|' | ':' | '/'))
        .filter(|part| !part.trim().is_empty())
        .map(normalize_token)
        .collect()
}

fn tokenize_path(path: &str) -> Vec<String> {
    let mut tokens = tokenize_path_segments(path);
    let basename = path
        .rsplit('/')
        .next()
        .map(normalize_token)
        .unwrap_or_default();
    if !basename.is_empty() {
        tokens.extend(split_keywords(&basename));
        tokens.push(basename);
    }
    dedupe_tokens(tokens)
}

fn tokenize_path_segments(path: &str) -> Vec<String> {
    path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(normalize_token)
        .collect()
}

fn dedupe_tokens(tokens: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();

    for token in tokens {
        if seen.insert(token.clone()) {
            ordered.push(token);
        }
    }

    ordered
}

#[cfg(test)]
mod tests {
    use super::search_sessions;
    use crate::models::{ScoreHints, SessionRecord, SessionType};

    fn make_session(
        session_id: &str,
        session_type: SessionType,
        cwd: &str,
        repo_name: &str,
        title: &str,
        score_hints: ScoreHints,
    ) -> SessionRecord {
        SessionRecord {
            session_id: session_id.into(),
            window_id: format!("w-{session_id}"),
            tab_id: format!("t-{session_id}"),
            miniaturized: false,
            title: title.into(),
            badge: None,
            cwd: Some(cwd.into()),
            repo_name: Some(repo_name.into()),
            foreground_command: Some(
                match session_type {
                    SessionType::Cc => "claude",
                    SessionType::Cx => "codex",
                    SessionType::Other => "zsh",
                }
                .into(),
            ),
            session_type,
            last_active_at: Some(1710000000),
            score_hints,
        }
    }

    #[test]
    fn excludes_hint_only_results_for_non_empty_query() {
        let sessions = vec![
            make_session(
                "match",
                SessionType::Cc,
                "/Users/me/work/repo-auth/auth",
                "repo-auth",
                "CC auth",
                ScoreHints {
                    exact_type: true,
                    exact_repo: true,
                    exact_path_segment: true,
                    recently_active: true,
                },
            ),
            make_session(
                "hint-only",
                SessionType::Cx,
                "/Users/me/work/repo-api/tests/api",
                "repo-api",
                "CX api tests",
                ScoreHints {
                    exact_type: true,
                    exact_repo: true,
                    exact_path_segment: true,
                    recently_active: true,
                },
            ),
        ];

        let results = search_sessions(&sessions, "cc auth", 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session.session_id, "match");
    }

    #[test]
    fn prefers_exact_repo_over_hyphenated_repo_subtoken_match() {
        let sessions = vec![
            make_session(
                "exact",
                SessionType::Cx,
                "~/personal/ccx",
                "ccx",
                "codex — ~/personal/ccx",
                ScoreHints::default(),
            ),
            make_session(
                "hyphenated",
                SessionType::Other,
                "~/personal/ccx-lens",
                "ccx-lens",
                "-zsh — ~/personal/ccx-lens",
                ScoreHints::default(),
            ),
        ];

        let results = search_sessions(&sessions, "ccx", 10);

        assert_eq!(results[0].session.session_id, "exact");
        assert!(results[0].score > results[1].score);
    }

}
