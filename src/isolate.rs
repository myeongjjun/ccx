use std::collections::HashSet;

use anyhow::{Context, Result};

use crate::models::{SearchResult, SessionRecord};

pub const DEFAULT_WINDOW_LIMIT: usize = 2;
pub const DEFAULT_GAP_RATIO: f64 = 0.7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedWindowAction {
    pub action: &'static str,
    pub session: SessionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsolatePlan {
    pub focus: SessionRecord,
    pub actions: Vec<PlannedWindowAction>,
}

pub fn build_isolate_plan(
    sessions: &[SessionRecord],
    results: Vec<SearchResult>,
    limit: usize,
    gap_ratio: f64,
    focus_window_id: Option<&str>,
) -> Result<IsolatePlan> {
    let keep_windows = build_isolate_window_results(results, limit, gap_ratio);
    let focus = focus_window_id
        .and_then(|window_id| {
            keep_windows
                .iter()
                .find(|result| result.session.window_id == window_id)
                .map(|result| result.session.clone())
        })
        .or_else(|| keep_windows.first().map(|result| result.session.clone()))
        .context("no matching sessions found")?;
    let keep_window_ids: HashSet<_> = keep_windows
        .iter()
        .map(|result| result.session.window_id.clone())
        .collect();
    let inventory = build_window_inventory(sessions);

    let mut actions = Vec::new();
    for result in keep_windows {
        let session = result.session;
        let action = if session.window_id == focus.window_id {
            if session.miniaturized {
                "restore-focus"
            } else {
                "focus"
            }
        } else if session.miniaturized {
            "restore"
        } else {
            "keep-visible"
        };

        actions.push(PlannedWindowAction { action, session });
    }

    for session in inventory {
        if keep_window_ids.contains(&session.window_id) {
            continue;
        }
        let action = if session.miniaturized {
            "leave-minimized"
        } else {
            "minimize"
        };
        actions.push(PlannedWindowAction { action, session });
    }

    Ok(IsolatePlan { focus, actions })
}

pub fn keep_window_ids(plan: &IsolatePlan) -> Vec<String> {
    plan.actions
        .iter()
        .filter(|action| action.action != "minimize" && action.action != "leave-minimized")
        .map(|action| action.session.window_id.clone())
        .collect()
}

pub fn kept_windows(plan: &IsolatePlan) -> Vec<SessionRecord> {
    plan.actions
        .iter()
        .filter(|action| action.action != "minimize" && action.action != "leave-minimized")
        .map(|action| action.session.clone())
        .collect()
}

fn build_ranked_window_results(results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen_windows = HashSet::new();
    let mut unique_windows = Vec::new();
    for result in results {
        if seen_windows.insert(result.session.window_id.clone()) {
            unique_windows.push(result);
        }
    }
    unique_windows
}

fn build_isolate_window_results(
    results: Vec<SearchResult>,
    limit: usize,
    gap_ratio: f64,
) -> Vec<SearchResult> {
    let unique_windows = build_ranked_window_results(results);
    if unique_windows.is_empty() {
        return unique_windows;
    }

    let seed_windows = build_seed_window_results(&unique_windows, limit, gap_ratio);
    let seed_window_ids: HashSet<_> = seed_windows
        .iter()
        .map(|result| result.session.window_id.clone())
        .collect();
    let seed_workset_keys: HashSet<_> = seed_windows
        .iter()
        .flat_map(expandable_workset_keys)
        .collect();

    unique_windows
        .into_iter()
        .filter(|result| {
            seed_window_ids.contains(&result.session.window_id)
                || shares_workset_identity(result, &seed_workset_keys)
        })
        .collect()
}

fn build_seed_window_results(
    unique_windows: &[SearchResult],
    limit: usize,
    gap_ratio: f64,
) -> Vec<SearchResult> {
    let best_score = unique_windows[0].score as f64;
    let cutoff_score = unique_windows
        .get(limit.saturating_sub(1))
        .map(|result| result.score)
        .unwrap_or_else(|| {
            unique_windows
                .last()
                .map(|result| result.score)
                .unwrap_or(0)
        });

    unique_windows
        .iter()
        .enumerate()
        .filter(|(index, result)| {
            *index == 0
                || (result.score >= cutoff_score && (result.score as f64) >= best_score * gap_ratio)
        })
        .map(|(_, result)| result.clone())
        .collect()
}

fn expandable_workset_keys(result: &SearchResult) -> Vec<String> {
    let mut keys = Vec::new();
    let matched_terms: HashSet<_> = result
        .matched_terms
        .iter()
        .map(|term| term.trim().to_lowercase())
        .collect();

    if let Some(repo_name) = result.session.repo_name.as_deref() {
        let repo_name = repo_name.trim().to_lowercase();
        if matched_terms.contains(&repo_name) {
            keys.push(format!("repo:{repo_name}"));
        }
    }
    if let Some(cwd) = result.session.cwd.as_deref() {
        let cwd = cwd.trim().to_lowercase();
        let basename = cwd.rsplit('/').next().unwrap_or_default().to_string();
        if !basename.is_empty() && matched_terms.contains(&basename) {
            keys.push(format!("cwd:{cwd}"));
        }
    }

    keys
}

fn shares_workset_identity(result: &SearchResult, seed_workset_keys: &HashSet<String>) -> bool {
    expandable_workset_keys(result)
        .into_iter()
        .any(|key| seed_workset_keys.contains(&key))
}

fn build_window_inventory(sessions: &[SessionRecord]) -> Vec<SessionRecord> {
    let mut seen_windows = HashSet::new();
    let mut unique_windows = Vec::new();
    for session in sessions {
        if seen_windows.insert(session.window_id.clone()) {
            unique_windows.push(session.clone());
        }
    }
    unique_windows
}

#[cfg(test)]
mod tests {
    use super::build_isolate_plan;
    use crate::models::{sample_session, ScoreHints, SearchResult, SessionRecord, SessionType};
    use std::collections::BTreeMap;

    #[test]
    fn isolate_restores_matching_windows_and_minimizes_others() {
        let mut best = sample_session();
        best.session_id = "best".into();
        best.window_id = "w1".into();
        best.miniaturized = true;

        let mut second = sample_session();
        second.session_id = "second".into();
        second.window_id = "w2".into();
        second.miniaturized = false;

        let mut other = sample_session();
        other.session_id = "other".into();
        other.window_id = "w3".into();
        other.miniaturized = false;

        let results = vec![
            SearchResult {
                session: best.clone(),
                score: 100,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: second.clone(),
                score: 90,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
        ];

        let plan = build_isolate_plan(&[best, second, other], results, 2, 0.7, None).expect("plan");
        let actions: Vec<_> = plan
            .actions
            .iter()
            .map(|action| (action.action, action.session.window_id.as_str()))
            .collect();

        assert_eq!(plan.focus.window_id, "w1");
        assert_eq!(
            actions,
            vec![
                ("restore-focus", "w1"),
                ("keep-visible", "w2"),
                ("minimize", "w3"),
            ]
        );
    }

    #[test]
    fn isolate_uses_top_window_scores_instead_of_session_duplicates() {
        let mut best = sample_session();
        best.session_id = "best".into();
        best.window_id = "w1".into();

        let mut same_window = sample_session();
        same_window.session_id = "same-window".into();
        same_window.window_id = "w1".into();

        let mut second = sample_session();
        second.session_id = "second".into();
        second.window_id = "w2".into();

        let mut third = sample_session();
        third.session_id = "third".into();
        third.window_id = "w3".into();

        let results = vec![
            SearchResult {
                session: best.clone(),
                score: 100,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: same_window,
                score: 95,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: second.clone(),
                score: 72,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: third.clone(),
                score: 71,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
        ];

        let plan = build_isolate_plan(&[best, second.clone(), third], results, 2, 0.7, None)
            .expect("plan");
        let kept: Vec<_> = plan
            .actions
            .iter()
            .filter(|action| action.action != "minimize" && action.action != "leave-minimized")
            .map(|action| action.session.window_id.as_str())
            .collect();

        assert_eq!(kept, vec!["w1", "w2"]);
    }

    #[test]
    fn isolate_drops_secondary_window_when_gap_is_too_large() {
        let mut best = sample_session();
        best.session_id = "best".into();
        best.window_id = "w1".into();

        let mut second = sample_session();
        second.session_id = "second".into();
        second.window_id = "w2".into();

        let mut third = sample_session();
        third.session_id = "third".into();
        third.window_id = "w3".into();

        let results = vec![
            SearchResult {
                session: best.clone(),
                score: 100,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: second.clone(),
                score: 60,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: third.clone(),
                score: 59,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
        ];

        let plan = build_isolate_plan(&[best, second, third], results, 2, 0.7, None).expect("plan");
        let actions: Vec<_> = plan
            .actions
            .iter()
            .map(|action| (action.action, action.session.window_id.as_str()))
            .collect();

        assert_eq!(
            actions,
            vec![("focus", "w1"), ("minimize", "w2"), ("minimize", "w3")]
        );
    }

    #[test]
    fn isolate_keeps_tied_windows_beyond_limit_cutoff() {
        let mut best = sample_session();
        best.session_id = "best".into();
        best.window_id = "w1".into();

        let mut second = sample_session();
        second.session_id = "second".into();
        second.window_id = "w2".into();

        let mut tied = sample_session();
        tied.session_id = "tied".into();
        tied.window_id = "w3".into();

        let mut other = sample_session();
        other.session_id = "other".into();
        other.window_id = "w4".into();

        let results = vec![
            SearchResult {
                session: best.clone(),
                score: 100,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: second.clone(),
                score: 70,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: tied.clone(),
                score: 70,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: other.clone(),
                score: 69,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
        ];

        let plan =
            build_isolate_plan(&[best, second, tied, other], results, 2, 0.7, None).expect("plan");
        let kept: Vec<_> = plan
            .actions
            .iter()
            .filter(|action| action.action != "minimize" && action.action != "leave-minimized")
            .map(|action| action.session.window_id.as_str())
            .collect();

        assert_eq!(kept, vec!["w1", "w2", "w3"]);
    }

    #[test]
    fn isolate_can_focus_selected_kept_window() {
        let mut best = sample_session();
        best.session_id = "best".into();
        best.window_id = "w1".into();

        let mut second = sample_session();
        second.session_id = "second".into();
        second.window_id = "w2".into();

        let results = vec![
            SearchResult {
                session: best.clone(),
                score: 100,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
            SearchResult {
                session: second.clone(),
                score: 90,
                matched_terms: Vec::new(),
                score_breakdown: BTreeMap::new(),
            },
        ];

        let plan =
            build_isolate_plan(&[best, second.clone()], results, 2, 0.7, Some("w2")).expect("plan");

        assert_eq!(plan.focus.window_id, "w2");
        assert_eq!(plan.actions[0].action, "keep-visible");
        assert_eq!(plan.actions[1].action, "focus");
    }

    #[test]
    fn isolate_keeps_same_workset_windows_beyond_seed_limit() {
        let ccx_best = session_with_identity("best", "w1", "ccx", "~/personal/ccx");
        let ccx_second = session_with_identity("second", "w2", "ccx", "~/personal/ccx");
        let ccx_third = session_with_identity("third", "w3", "ccx", "~/personal/ccx");
        let other = session_with_identity("other", "w4", "clickhouse", "~/clickhouse/ClickHouse");

        let results = vec![
            result_for_with_matches(ccx_best.clone(), 100, &["ccx"]),
            result_for_with_matches(ccx_second.clone(), 95, &["ccx"]),
            result_for_with_matches(ccx_third.clone(), 90, &["ccx"]),
            result_for_with_matches(other.clone(), 80, &["clickhouse"]),
        ];

        let plan = build_isolate_plan(
            &[ccx_best, ccx_second, ccx_third, other],
            results,
            2,
            0.7,
            None,
        )
        .expect("plan");
        let kept: Vec<_> = plan
            .actions
            .iter()
            .filter(|action| action.action != "minimize" && action.action != "leave-minimized")
            .map(|action| action.session.window_id.as_str())
            .collect();

        assert_eq!(kept, vec!["w1", "w2", "w3"]);
    }

    fn result_for_with_matches(
        session: SessionRecord,
        score: i64,
        matched_terms: &[&str],
    ) -> SearchResult {
        SearchResult {
            session,
            score,
            matched_terms: matched_terms.iter().map(|term| term.to_string()).collect(),
            score_breakdown: BTreeMap::new(),
        }
    }

    fn session_with_identity(
        session_id: &str,
        window_id: &str,
        repo_name: &str,
        cwd: &str,
    ) -> SessionRecord {
        SessionRecord {
            session_id: session_id.into(),
            window_id: window_id.into(),
            tab_id: format!("t-{session_id}"),
            miniaturized: false,
            title: format!("Session {session_id}"),
            badge: None,
            cwd: Some(cwd.into()),
            repo_name: Some(repo_name.into()),
            foreground_command: Some("codex".into()),
            session_type: SessionType::Cx,
            last_active_at: None,
            score_hints: ScoreHints::default(),
        }
    }
}
