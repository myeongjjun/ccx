mod cache;
mod classifier;
mod debug;
mod isolate;
mod isolate_tui;
mod iterm;
mod models;
mod search;
mod terminal_ui;
mod tui;

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use isolate::{
    build_isolate_plan, keep_window_ids, IsolatePlan, DEFAULT_GAP_RATIO, DEFAULT_WINDOW_LIMIT,
};
use models::{SearchResult, SessionRecord};
use search::MAX_SCAN_RESULTS;

#[derive(Parser, Debug)]
#[command(name = "ccx")]
#[command(version)]
#[command(about = "Rust implementation of the ccx iTerm2 session launcher")]
#[command(
    after_help = "Running `ccx` opens the interactive launcher.\nRunning `ccx <query>` is shorthand for `ccx isolate <query>`."
)]
struct Cli {
    #[arg(long, global = true)]
    debug: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print a sample session schema.
    Schema,
    /// Search normalized session snapshots.
    Search {
        query: String,
        #[arg(long)]
        sessions: Option<PathBuf>,
        #[arg(long)]
        cache: bool,
        #[arg(long)]
        cache_path: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long)]
        refresh_now: bool,
    },
    /// Collect sessions from iTerm2 and optionally persist them as a snapshot.
    Collect {
        #[arg(long)]
        raw: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Run the terminal-native interactive launcher.
    Tui {
        #[arg(long)]
        sessions: Option<PathBuf>,
        #[arg(long)]
        cache: bool,
        #[arg(long)]
        cache_path: Option<PathBuf>,
        #[arg(long, default_value_t = 8)]
        limit: usize,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long)]
        refresh_now: bool,
    },
    /// Run the isolate preview UI and apply the selected workset.
    IsolateTui {
        #[arg(long)]
        sessions: Option<PathBuf>,
        #[arg(long)]
        cache: bool,
        #[arg(long)]
        cache_path: Option<PathBuf>,
        #[arg(long, default_value_t = DEFAULT_WINDOW_LIMIT)]
        limit: usize,
        #[arg(long, default_value_t = DEFAULT_GAP_RATIO)]
        gap_ratio: f64,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long)]
        refresh_now: bool,
    },
    /// Pick the best matching session and optionally focus it.
    Pick {
        query: String,
        #[arg(long)]
        sessions: Option<PathBuf>,
        #[arg(long)]
        cache: bool,
        #[arg(long)]
        cache_path: Option<PathBuf>,
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long)]
        refresh_now: bool,
    },
    /// Promote a workset by focusing ranked windows from lowest to highest score.
    Promote {
        query: String,
        #[arg(long)]
        sessions: Option<PathBuf>,
        #[arg(long)]
        cache: bool,
        #[arg(long)]
        cache_path: Option<PathBuf>,
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        per_session: bool,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long)]
        refresh_now: bool,
    },
    /// Leave the matching workset visible and minimize unrelated iTerm2 windows.
    Isolate {
        query: String,
        #[arg(long)]
        sessions: Option<PathBuf>,
        #[arg(long)]
        cache: bool,
        #[arg(long)]
        cache_path: Option<PathBuf>,
        #[arg(long, default_value_t = DEFAULT_WINDOW_LIMIT)]
        limit: usize,
        #[arg(long, default_value_t = DEFAULT_GAP_RATIO)]
        gap_ratio: f64,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long)]
        refresh_now: bool,
    },
    /// Print the AppleScript used to focus a session from a JSON file.
    FocusScript {
        #[arg(long)]
        session: PathBuf,
    },
    #[command(external_subcommand)]
    ExternalQuery(Vec<String>),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    debug::init(cli.debug);

    match cli.command {
        None => {
            let sessions = resolve_session_source(None, false, None, RefreshBehavior::Auto)?;
            if let Some(selected) = tui::run_launcher(&sessions, 8)? {
                focus_session_action(&selected, "launcher")?;
            }
        }
        Some(Command::ExternalQuery(args)) => {
            let query = args.join(" ").trim().to_string();
            if query.is_empty() {
                anyhow::bail!("query shorthand requires at least one term");
            }
            let sessions = resolve_session_source(None, false, None, RefreshBehavior::Auto)?;
            let results =
                search::search_sessions(&sessions, &query, sessions.len().min(MAX_SCAN_RESULTS));
            let plan = build_isolate_plan(
                &sessions,
                results,
                DEFAULT_WINDOW_LIMIT,
                DEFAULT_GAP_RATIO,
                None,
            )?;
            debug::log(format!(
                "isolate shorthand query={query:?} keep_windows={} focus_window_id={} gap_ratio={}",
                keep_window_ids(&plan).len(),
                plan.focus.window_id,
                DEFAULT_GAP_RATIO
            ));
            run_isolate_action(&plan, "isolate shorthand")?;
        }
        Some(Command::Schema) => {
            let sample = models::sample_session();
            println!("{}", serde_json::to_string_pretty(&sample)?);
        }
        Some(Command::Search {
            query,
            sessions,
            cache,
            cache_path,
            limit,
            json,
            explain,
            no_refresh,
            refresh_now,
        }) => {
            let sessions = resolve_session_source(
                sessions,
                cache,
                cache_path,
                RefreshBehavior::from_flags(no_refresh, refresh_now)?,
            )?;
            let results = search::search_sessions(&sessions, &query, limit);
            debug::log(format!(
                "search query={query:?} sessions={} results={} limit={limit}",
                sessions.len(),
                results.len()
            ));
            if json {
                let payload: Vec<_> = results
                    .iter()
                    .map(|result| result.as_json(explain))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else if results.is_empty() {
                println!("No matching sessions found.");
            } else {
                for result in results {
                    println!("{}", result.render_line(explain));
                }
            }
        }
        Some(Command::Collect {
            raw,
            dry_run,
            output,
        }) => {
            if dry_run {
                debug::log("collect dry-run requested");
                println!("{}", iterm::build_collection_script());
            } else if raw {
                let sessions = iterm::collect_raw_sessions()?;
                debug::log(format!("collect raw returned {} sessions", sessions.len()));
                println!("{}", serde_json::to_string_pretty(&sessions)?);
            } else {
                let sessions = iterm::collect_sessions()?;
                let output_path = output.unwrap_or_else(cache::default_cache_path);
                debug::log(format!(
                    "collect normalized {} sessions into {}",
                    sessions.len(),
                    output_path.display()
                ));
                cache::write_sessions(&output_path, &sessions)?;
                println!("{}", serde_json::to_string_pretty(&sessions)?);
            }
        }
        Some(Command::Tui {
            sessions,
            cache,
            cache_path,
            limit,
            no_refresh,
            refresh_now,
        }) => {
            let sessions = resolve_session_source(
                sessions,
                cache,
                cache_path,
                RefreshBehavior::from_flags(no_refresh, refresh_now)?,
            )?;
            if let Some(selected) = tui::run_launcher(&sessions, limit)? {
                focus_session_action(&selected, "tui")?;
            }
        }
        Some(Command::IsolateTui {
            sessions,
            cache,
            cache_path,
            limit,
            gap_ratio,
            no_refresh,
            refresh_now,
        }) => {
            if limit == 0 {
                anyhow::bail!("--limit must be at least 1");
            }
            if !(0.0..=1.0).contains(&gap_ratio) {
                anyhow::bail!("--gap-ratio must be between 0.0 and 1.0");
            }
            let sessions = resolve_session_source(
                sessions,
                cache,
                cache_path,
                RefreshBehavior::from_flags(no_refresh, refresh_now)?,
            )?;
            if let Some(plan) = isolate_tui::run_launcher(&sessions, limit, gap_ratio)? {
                run_isolate_action(&plan, "isolate-tui")?;
            }
        }
        Some(Command::Pick {
            query,
            sessions,
            cache,
            cache_path,
            limit,
            dry_run,
            no_refresh,
            refresh_now,
        }) => {
            let sessions = resolve_session_source(
                sessions,
                cache,
                cache_path,
                RefreshBehavior::from_flags(no_refresh, refresh_now)?,
            )?;
            let results = search::search_sessions(&sessions, &query, limit);
            let best = results
                .into_iter()
                .next()
                .context("no matching sessions found")?;
            debug::log(format!(
                "pick query={query:?} selected session_id={} window_id={}",
                best.session.session_id, best.session.window_id
            ));

            if dry_run {
                println!("{}", serde_json::to_string_pretty(&best.session)?);
                println!();
                println!("{}", iterm::build_focus_script(&best.session));
            } else {
                focus_session_action(&best.session, "pick")?;
            }
        }
        Some(Command::Promote {
            query,
            sessions,
            cache,
            cache_path,
            limit,
            dry_run,
            quiet,
            per_session,
            no_refresh,
            refresh_now,
        }) => {
            let sessions = resolve_session_source(
                sessions,
                cache,
                cache_path,
                RefreshBehavior::from_flags(no_refresh, refresh_now)?,
            )?;
            let results = search::search_sessions(&sessions, &query, limit);
            if results.is_empty() {
                anyhow::bail!("no matching sessions found");
            }

            let promote_order = build_promote_order(results, per_session);
            debug::log(format!(
                "promote query={query:?} windows={} per_session={per_session} quiet={quiet}",
                promote_order.len()
            ));

            if dry_run {
                let payload: Vec<_> = promote_order
                    .iter()
                    .enumerate()
                    .map(|(index, session)| {
                        serde_json::json!({
                            "step": index + 1,
                            "mode": if quiet && index + 1 != promote_order.len() { "quiet" } else { "visible" },
                            "session": session,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                for (index, session) in promote_order.iter().enumerate() {
                    if debug::enabled() {
                        let status = if per_session {
                            if quiet && index + 1 != promote_order.len() {
                                iterm::quietly_focus_session(session)?
                            } else {
                                iterm::focus_session(session)?
                            }
                        } else {
                            iterm::focus_window(&session.window_id)?
                        };
                        report_action_status(format!(
                            "promote session_id={} window_id={} step={} status={status}",
                            session.session_id,
                            session.window_id,
                            index + 1
                        ));
                    } else {
                        if per_session {
                            if quiet && index + 1 != promote_order.len() {
                                iterm::quietly_focus_session_async(session)?
                            } else {
                                iterm::focus_session_async(session)?
                            }
                        } else {
                            iterm::focus_window_async(&session.window_id)?
                        }
                    }
                }
            }
        }
        Some(Command::Isolate {
            query,
            sessions,
            cache,
            cache_path,
            limit,
            gap_ratio,
            dry_run,
            no_refresh,
            refresh_now,
        }) => {
            if limit == 0 {
                anyhow::bail!("--limit must be at least 1");
            }
            if !(0.0..=1.0).contains(&gap_ratio) {
                anyhow::bail!("--gap-ratio must be between 0.0 and 1.0");
            }
            let sessions = resolve_session_source(
                sessions,
                cache,
                cache_path,
                RefreshBehavior::from_flags(no_refresh, refresh_now)?,
            )?;
            let results =
                search::search_sessions(&sessions, &query, sessions.len().min(MAX_SCAN_RESULTS));
            let plan = build_isolate_plan(&sessions, results, limit, gap_ratio, None)?;
            debug::log(format!(
                "isolate query={query:?} keep_windows={} focus_window_id={} gap_ratio={gap_ratio}",
                keep_window_ids(&plan).len(),
                plan.focus.window_id
            ));

            if dry_run {
                let payload: Vec<_> = plan
                    .actions
                    .iter()
                    .enumerate()
                    .map(|(index, action)| {
                        serde_json::json!({
                            "step": index + 1,
                            "action": action.action,
                            "window_id": action.session.window_id,
                            "session": action.session,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                run_isolate_action(&plan, "isolate")?;
            }
        }
        Some(Command::FocusScript { session }) => {
            let session = cache::load_single_session(&session)?;
            println!("{}", iterm::build_focus_script(&session));
        }
    }

    Ok(())
}

fn build_promote_order(results: Vec<SearchResult>, per_session: bool) -> Vec<SessionRecord> {
    if per_session {
        return results
            .into_iter()
            .map(|result| result.session)
            .rev()
            .collect();
    }

    build_ranked_window_results(results)
        .into_iter()
        .map(|result| result.session)
        .rev()
        .collect()
}

fn build_ranked_window_results(results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen_windows = std::collections::HashSet::new();
    let mut unique_windows = Vec::new();
    for result in results {
        if seen_windows.insert(result.session.window_id.clone()) {
            unique_windows.push(result);
        }
    }
    unique_windows
}

#[derive(Debug, Clone)]
struct LiveWindowState {
    window_id: String,
    title: String,
    miniaturized: bool,
}

fn apply_isolate_plan(plan: &IsolatePlan) -> Result<String> {
    let keep_ids = keep_window_ids(plan);

    let before = if debug::enabled() {
        match collect_live_window_states() {
            Ok(states) => {
                log_live_window_snapshot("before", &keep_ids, &states);
                Some(states)
            }
            Err(error) => {
                debug::log(format!(
                    "failed to collect live state before isolate: {error}"
                ));
                None
            }
        }
    } else {
        None
    };

    let report = iterm::isolate_windows_report(&keep_ids, &plan.focus.window_id)?;
    debug::log(format!("isolate apply status={}", report.status));

    if debug::enabled() {
        let after: Vec<_> = report
            .windows
            .iter()
            .map(|window| LiveWindowState {
                window_id: window.window_id.clone(),
                title: window.title.clone(),
                miniaturized: window.miniaturized,
            })
            .collect();
        log_live_window_snapshot("after", &keep_ids, &after);
        if let Some(before) = before.as_ref() {
            log_live_window_diff(before, &after, &keep_ids);
        }
        log_live_window_mismatches(&after, &keep_ids);
        if !report.mismatches.is_empty() {
            debug::log(format!(
                "same-process mismatch ids=[{}]",
                report.mismatches.join(",")
            ));
        }
        for error in &report.errors {
            debug::log(format!(
                "window error window_id={} phase={} message={}",
                error.window_id, error.phase, error.message
            ));
        }
    }

    Ok(report.status)
}

fn report_action_status(message: impl AsRef<str>) {
    if debug::enabled() {
        debug::log(message);
    }
}

fn focus_session_action(session: &SessionRecord, label: &str) -> Result<()> {
    if debug::enabled() {
        let status = iterm::focus_session(session)?;
        report_action_status(format!(
            "{label} focus session_id={} window_id={} status={status}",
            session.session_id, session.window_id
        ));
    } else {
        iterm::focus_session_async(session)?;
    }
    Ok(())
}

fn run_isolate_action(plan: &IsolatePlan, label: &str) -> Result<()> {
    if debug::enabled() {
        let status = apply_isolate_plan(plan)?;
        report_action_status(format!(
            "{label} focus_window_id={} status={status}",
            plan.focus.window_id
        ));
    } else {
        iterm::isolate_windows_async(&keep_window_ids(plan), &plan.focus.window_id)?;
    }
    Ok(())
}

fn collect_live_window_states() -> Result<Vec<LiveWindowState>> {
    Ok(iterm::collect_window_states()?
        .into_iter()
        .map(|window| LiveWindowState {
            window_id: window.window_id,
            title: window.title,
            miniaturized: window.miniaturized,
        })
        .collect())
}

fn log_live_window_snapshot(label: &str, keep_ids: &[String], states: &[LiveWindowState]) {
    let keep: HashSet<_> = keep_ids.iter().cloned().collect();
    let visible: Vec<_> = states
        .iter()
        .filter(|state| !state.miniaturized)
        .map(|state| render_live_window(state, keep.contains(&state.window_id)))
        .collect();
    let minimized: Vec<_> = states
        .iter()
        .filter(|state| state.miniaturized)
        .map(|state| render_live_window(state, keep.contains(&state.window_id)))
        .collect();

    debug::log(format!(
        "live {label} visible={} [{}]",
        visible.len(),
        visible.join(" | ")
    ));
    debug::log(format!(
        "live {label} minimized={} [{}]",
        minimized.len(),
        minimized.join(" | ")
    ));
}

fn log_live_window_diff(
    before: &[LiveWindowState],
    after: &[LiveWindowState],
    keep_ids: &[String],
) {
    let keep: HashSet<_> = keep_ids.iter().cloned().collect();
    let before_map: BTreeMap<_, _> = before
        .iter()
        .map(|state| (state.window_id.clone(), state.miniaturized))
        .collect();
    let changes: Vec<_> = after
        .iter()
        .filter_map(|state| {
            let before_miniaturized = before_map.get(&state.window_id)?;
            if *before_miniaturized == state.miniaturized {
                return None;
            }
            Some(format!(
                "{} {} -> {} {}",
                state.window_id,
                if *before_miniaturized {
                    "minimized"
                } else {
                    "visible"
                },
                if state.miniaturized {
                    "minimized"
                } else {
                    "visible"
                },
                if keep.contains(&state.window_id) {
                    "(keep)"
                } else {
                    "(drop)"
                }
            ))
        })
        .collect();

    if changes.is_empty() {
        debug::log("live diff changed_windows=0");
    } else {
        debug::log(format!(
            "live diff changed_windows={} [{}]",
            changes.len(),
            changes.join(" | ")
        ));
    }
}

fn log_live_window_mismatches(states: &[LiveWindowState], keep_ids: &[String]) {
    let keep: HashSet<_> = keep_ids.iter().cloned().collect();
    let mismatches: Vec<_> = states
        .iter()
        .filter_map(|state| {
            let should_be_visible = keep.contains(&state.window_id);
            let is_visible = !state.miniaturized;
            if should_be_visible == is_visible {
                None
            } else {
                Some(format!(
                    "{} expected={} actual={} {}",
                    state.window_id,
                    if should_be_visible {
                        "visible"
                    } else {
                        "minimized"
                    },
                    if is_visible { "visible" } else { "minimized" },
                    state.title
                ))
            }
        })
        .collect();

    if mismatches.is_empty() {
        debug::log("live mismatch count=0");
    } else {
        debug::log(format!(
            "live mismatch count={} [{}]",
            mismatches.len(),
            mismatches.join(" | ")
        ));
    }
}

fn render_live_window(state: &LiveWindowState, should_keep: bool) -> String {
    format!(
        "{}:{}:{}:{}",
        state.window_id,
        if state.miniaturized { "min" } else { "vis" },
        if should_keep { "keep" } else { "drop" },
        state.title
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshBehavior {
    Auto,
    NoRefresh,
    RefreshNow,
}

impl RefreshBehavior {
    fn from_flags(no_refresh: bool, refresh_now: bool) -> Result<Self> {
        if no_refresh && refresh_now {
            anyhow::bail!("--no-refresh and --refresh-now cannot be used together");
        }
        if refresh_now {
            Ok(Self::RefreshNow)
        } else if no_refresh {
            Ok(Self::NoRefresh)
        } else {
            Ok(Self::Auto)
        }
    }
}

fn resolve_session_source(
    sessions: Option<PathBuf>,
    cache: bool,
    cache_path: Option<PathBuf>,
    refresh_behavior: RefreshBehavior,
) -> Result<Vec<SessionRecord>> {
    if sessions.is_some() && cache {
        anyhow::bail!("--sessions and --cache cannot be used together");
    }
    if sessions.is_some() && refresh_behavior != RefreshBehavior::Auto {
        anyhow::bail!("refresh flags can only be used with the managed cache");
    }
    if let Some(path) = sessions {
        debug::log(format!("loading sessions from file {}", path.display()));
        return cache::load_sessions(&path)
            .with_context(|| format!("failed to load session file: {}", path.display()));
    }

    let path = if cache || cache_path.is_some() {
        cache_path.unwrap_or_else(cache::default_cache_path)
    } else {
        cache::default_cache_path()
    };
    debug::log(format!(
        "loading managed cache {} refresh_behavior={refresh_behavior:?}",
        path.display()
    ));

    ensure_cache_ready(&path, refresh_behavior)?;

    cache::load_sessions(&path)
        .with_context(|| format!("failed to load cache file: {}", path.display()))
}

fn ensure_cache_ready(path: &Path, refresh_behavior: RefreshBehavior) -> Result<()> {
    if !cache::cache_exists(path) || refresh_behavior == RefreshBehavior::RefreshNow {
        debug::log(format!(
            "refreshing cache now path={} reason={}",
            path.display(),
            if !cache::cache_exists(path) {
                "missing"
            } else {
                "refresh-now"
            }
        ));
        refresh_cache_now(path)?;
        return Ok(());
    }

    if refresh_behavior == RefreshBehavior::Auto
        && cache::cache_is_stale(path, cache::DEFAULT_CACHE_STALE_AFTER)?
    {
        debug::log(format!(
            "cache stale, spawning background refresh for {}",
            path.display()
        ));
        if let Err(error) = spawn_background_refresh(path) {
            eprintln!("warning: failed to spawn background refresh: {error}");
        }
    } else {
        debug::log(format!("cache ready without refresh {}", path.display()));
    }

    Ok(())
}

fn refresh_cache_now(path: &Path) -> Result<()> {
    let sessions = iterm::collect_sessions()?;
    cache::write_sessions(path, &sessions)
}

fn spawn_background_refresh(path: &Path) -> Result<()> {
    let exe = std::env::current_exe().context("failed to resolve current executable")?;
    let mut command = ProcessCommand::new(exe);
    command
        .arg("collect")
        .arg("--output")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
        .spawn()
        .context("failed to spawn background refresh")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_promote_order, Cli, Command};
    use crate::models::SearchResult;
    use clap::Parser;

    fn result(session_id: &str, window_id: &str, score: i64) -> SearchResult {
        let mut session = crate::models::sample_session();
        session.session_id = session_id.into();
        session.window_id = window_id.into();
        SearchResult {
            session,
            score,
            matched_terms: Vec::new(),
            score_breakdown: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn promote_defaults_to_unique_windows() {
        let results = vec![
            result("best-w1", "w1", 100),
            result("best-w2", "w2", 90),
            result("second-w1", "w1", 80),
        ];

        let order = build_promote_order(results, false);
        let ids: Vec<_> = order
            .iter()
            .map(|session| session.session_id.as_str())
            .collect();

        assert_eq!(ids, vec!["best-w2", "best-w1"]);
    }

    #[test]
    fn promote_can_keep_session_level_order() {
        let results = vec![
            result("best-w1", "w1", 100),
            result("best-w2", "w2", 90),
            result("second-w1", "w1", 80),
        ];

        let order = build_promote_order(results, true);
        let ids: Vec<_> = order
            .iter()
            .map(|session| session.session_id.as_str())
            .collect();

        assert_eq!(ids, vec!["second-w1", "best-w2", "best-w1"]);
    }

    #[test]
    fn cli_defaults_to_launcher_when_no_subcommand_is_given() {
        let cli = Cli::try_parse_from(["ccx"]).expect("cli should parse");
        assert!(cli.command.is_none());
    }

    #[test]
    fn cli_still_parses_explicit_subcommands() {
        let cli = Cli::try_parse_from(["ccx", "isolate-tui"]).expect("cli should parse");
        assert!(matches!(cli.command, Some(Command::IsolateTui { .. })));
    }

    #[test]
    fn cli_treats_bare_query_as_isolate_shorthand() {
        let cli = Cli::try_parse_from(["ccx", "ccx"]).expect("cli should parse");
        assert!(matches!(
            cli.command,
            Some(Command::ExternalQuery(args)) if args == vec!["ccx"]
        ));
    }
}
