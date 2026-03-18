use std::io;

use anyhow::Result;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;

use crate::isolate::{build_isolate_plan, kept_windows, IsolatePlan};
use crate::iterm;
use crate::models::SessionRecord;
use crate::search::{search_sessions, MAX_SCAN_RESULTS};
use crate::terminal_ui::TerminalCleanupGuard;

#[derive(Debug)]
struct TuiState {
    query: String,
    selected_index: usize,
    contents_session_id: Option<String>,
    contents_preview: String,
}

#[derive(Debug)]
struct PreviewState {
    plan: Option<IsolatePlan>,
    kept_windows: Vec<SessionRecord>,
    summary: String,
    contents_title: String,
    contents_preview: String,
}

pub fn run_launcher(
    sessions: &[SessionRecord],
    limit: usize,
    gap_ratio: f64,
) -> Result<Option<IsolatePlan>> {
    let mut guard = TerminalCleanupGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState {
        query: String::new(),
        selected_index: 0,
        contents_session_id: None,
        contents_preview: "Select a visible window to load live contents.".into(),
    };

    let result = loop {
        let preview = build_preview(sessions, &mut state, limit, gap_ratio)?;
        terminal.draw(|frame| render(frame, &state, &preview))?;

        if let Event::Key(event) = read()? {
            if let Some(outcome) = handle_key(&mut state, &preview, event) {
                break outcome;
            }
        }
    };

    guard.finish()?;
    terminal.show_cursor()?;
    Ok(result)
}

fn build_preview(
    sessions: &[SessionRecord],
    state: &mut TuiState,
    limit: usize,
    gap_ratio: f64,
) -> Result<PreviewState> {
    if state.query.trim().is_empty() {
        state.selected_index = 0;
        return Ok(PreviewState {
            plan: None,
            kept_windows: Vec::new(),
            summary: "Type a query to preview isolate.".into(),
            contents_title: "Visible Contents".into(),
            contents_preview: state.contents_preview.clone(),
        });
    }

    let results = search_sessions(sessions, &state.query, sessions.len().min(MAX_SCAN_RESULTS));
    if results.is_empty() {
        state.selected_index = 0;
        return Ok(PreviewState {
            plan: None,
            kept_windows: Vec::new(),
            summary: "No matching windows for this query.".into(),
            contents_title: "Visible Contents".into(),
            contents_preview: state.contents_preview.clone(),
        });
    }

    let default_plan = build_isolate_plan(sessions, results.clone(), limit, gap_ratio, None)?;
    let windows = kept_windows(&default_plan);
    state.selected_index = state.selected_index.min(windows.len().saturating_sub(1));
    let selected_focus = windows
        .get(state.selected_index)
        .map(|session| session.window_id.as_str());
    let plan = build_isolate_plan(sessions, results, limit, gap_ratio, selected_focus)?;
    let kept = kept_windows(&plan);
    refresh_contents_preview(state, kept.get(state.selected_index));
    let focus_title = kept
        .get(state.selected_index)
        .map(|session| session.title.clone())
        .unwrap_or_else(|| plan.focus.title.clone());

    Ok(PreviewState {
        plan: Some(plan),
        kept_windows: kept,
        summary: format!(
            "{} window(s) visible after isolate. Enter applies. Focus anchor: {}",
            windows.len(),
            focus_title
        ),
        contents_title: format!("Visible Contents: {}", focus_title),
        contents_preview: state.contents_preview.clone(),
    })
}

fn refresh_contents_preview(state: &mut TuiState, selected: Option<&SessionRecord>) {
    let Some(selected) = selected else {
        state.contents_session_id = None;
        state.contents_preview = "No visible window selected.".into();
        return;
    };

    if state.contents_session_id.as_deref() == Some(selected.session_id.as_str()) {
        return;
    }

    state.contents_session_id = Some(selected.session_id.clone());
    state.contents_preview = match iterm::session_contents(&selected.session_id) {
        Ok(contents) if !contents.trim().is_empty() => trim_contents_preview(&contents, 32),
        Ok(_) => "No visible contents returned for this session.".into(),
        Err(error) => format!("Unable to load live contents: {error}"),
    };
}

fn handle_key(
    state: &mut TuiState,
    preview: &PreviewState,
    event: KeyEvent,
) -> Option<Option<IsolatePlan>> {
    match (event.code, event.modifiers) {
        (KeyCode::Esc, _) => Some(None),
        (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
            if !preview.kept_windows.is_empty() {
                state.selected_index = state.selected_index.saturating_sub(1);
            }
            None
        }
        (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
            if !preview.kept_windows.is_empty() {
                state.selected_index =
                    (state.selected_index + 1).min(preview.kept_windows.len().saturating_sub(1));
            }
            None
        }
        (KeyCode::Backspace, _) => {
            state.query.pop();
            state.selected_index = 0;
            None
        }
        (KeyCode::Enter, _) => preview.plan.clone().map(Some),
        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            state.query.push(ch);
            state.selected_index = 0;
            None
        }
        _ => None,
    }
}

fn render(frame: &mut ratatui::Frame<'_>, state: &TuiState, preview: &PreviewState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    let header = Paragraph::new(vec![
        Line::from("ccx isolate-tui"),
        Line::from("Esc exit  Up/Down move focus anchor  Enter isolate current preview"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Mode"));
    frame.render_widget(header, chunks[0]);

    let query = Paragraph::new(state.query.as_str())
        .block(Block::default().borders(Borders::ALL).title("Query"))
        .wrap(Wrap { trim: false });
    frame.render_widget(query, chunks[1]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[2]);

    let items: Vec<ListItem> = if preview.kept_windows.is_empty() {
        vec![ListItem::new("No visible workset preview yet.")]
    } else {
        preview
            .kept_windows
            .iter()
            .map(|session| {
                ListItem::new(vec![
                    Line::from(Span::raw(session.title.as_str())),
                    Line::from(format!(
                        "{}  {}",
                        session.repo_name.as_deref().unwrap_or("-"),
                        session.cwd.as_deref().unwrap_or("-")
                    )),
                ])
            })
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Visible Workset"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");
    let mut list_state = ListState::default();
    if !preview.kept_windows.is_empty() {
        list_state.select(Some(state.selected_index));
    }
    frame.render_stateful_widget(list, body[0], &mut list_state);

    let contents_panel = Paragraph::new(preview.contents_preview.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(preview.contents_title.as_str()),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(contents_panel, body[1]);

    let footer = Paragraph::new(preview.summary.as_str())
        .block(Block::default().borders(Borders::TOP).title("Summary"));
    frame.render_widget(footer, chunks[3]);
}

fn trim_contents_preview(contents: &str, max_lines: usize) -> String {
    let lines: Vec<_> = contents.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::{handle_key, PreviewState, TuiState};

    fn state() -> TuiState {
        TuiState {
            query: String::new(),
            selected_index: 0,
            contents_session_id: None,
            contents_preview: String::new(),
        }
    }

    fn preview() -> PreviewState {
        PreviewState {
            plan: None,
            kept_windows: Vec::new(),
            summary: String::new(),
            contents_title: String::new(),
            contents_preview: String::new(),
        }
    }

    #[test]
    fn q_can_start_a_query() {
        let mut state = state();
        let outcome = handle_key(
            &mut state,
            &preview(),
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('q'),
                crossterm::event::KeyModifiers::NONE,
            ),
        );

        assert!(outcome.is_none());
        assert_eq!(state.query, "q");
    }
}
