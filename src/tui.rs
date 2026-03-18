use std::io::{self, Write};

use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::{Attribute, SetAttribute};
use crossterm::terminal::{size, Clear, ClearType};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::models::SessionRecord;
use crate::search::search_sessions;
use crate::terminal_ui::TerminalCleanupGuard;

#[derive(Debug)]
struct TuiState {
    query: String,
    selected_index: usize,
    limit: usize,
    status_message: String,
}

pub fn run_launcher(sessions: &[SessionRecord], limit: usize) -> Result<Option<SessionRecord>> {
    let mut state = TuiState {
        query: String::new(),
        selected_index: 0,
        limit,
        status_message: "Type to search. Enter focuses the selected session.".into(),
    };

    let mut guard = TerminalCleanupGuard::enter()?;
    let mut stdout = io::stdout();

    let result = loop {
        let results = search_sessions(sessions, &state.query, state.limit);
        render(&mut stdout, &state, &results)?;

        if let Event::Key(event) = read()? {
            if let Some(outcome) = handle_key(&mut state, &results, event)? {
                break outcome;
            }
        }
    };

    guard.finish()?;
    Ok(result)
}

fn handle_key(
    state: &mut TuiState,
    results: &[crate::models::SearchResult],
    event: KeyEvent,
) -> Result<Option<Option<SessionRecord>>> {
    match (event.code, event.modifiers) {
        (KeyCode::Esc, _) => {
            return Ok(Some(None));
        }
        (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
            if !results.is_empty() {
                state.selected_index = state.selected_index.saturating_sub(1);
            }
        }
        (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
            if !results.is_empty() {
                state.selected_index =
                    (state.selected_index + 1).min(results.len().saturating_sub(1));
            }
        }
        (KeyCode::Backspace, _) => {
            state.query.pop();
            state.selected_index = 0;
        }
        (KeyCode::Enter, _) => {
            if let Some(selected) = results.get(state.selected_index) {
                return Ok(Some(Some(selected.session.clone())));
            }
        }
        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            state.query.push(ch);
            state.selected_index = 0;
        }
        _ => {}
    }

    Ok(None)
}

fn render(
    stdout: &mut io::Stdout,
    state: &TuiState,
    results: &[crate::models::SearchResult],
) -> Result<()> {
    let (width, height) = size()?;
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

    write_line(stdout, 0, width, "ccx TUI")?;
    write_line(
        stdout,
        1,
        width,
        "Esc exit  Up/Down or Ctrl-p/Ctrl-n move  Enter focus",
    )?;
    write_line(stdout, 2, width, &format!("Query: {}", state.query))?;
    write_line(stdout, 3, width, "")?;

    if results.is_empty() {
        write_line(stdout, 4, width, "No matching sessions.")?;
    } else {
        for (index, result) in results
            .iter()
            .take(height.saturating_sub(6) as usize)
            .enumerate()
        {
            let row = (4 + index) as u16;
            let session = &result.session;
            let line = format!(
                "{} {:<2} {:<18} {} :: {}",
                if index == state.selected_index {
                    ">"
                } else {
                    " "
                },
                session.session_type.as_label(),
                session.repo_name.as_deref().unwrap_or("-"),
                session.cwd.as_deref().unwrap_or("-"),
                session.title
            );
            write_result_line(stdout, row, width, &line, index == state.selected_index)?;
        }
    }

    write_line(
        stdout,
        height.saturating_sub(1),
        width,
        &state.status_message,
    )?;
    stdout.flush()?;
    Ok(())
}

fn write_line(stdout: &mut io::Stdout, row: u16, width: u16, text: &str) -> Result<()> {
    queue!(stdout, MoveTo(0, row), Clear(ClearType::CurrentLine))?;
    write!(stdout, "{}", fit_to_width(text, width as usize))?;
    Ok(())
}

fn write_result_line(
    stdout: &mut io::Stdout,
    row: u16,
    width: u16,
    text: &str,
    selected: bool,
) -> Result<()> {
    queue!(stdout, MoveTo(0, row), Clear(ClearType::CurrentLine))?;
    if selected {
        queue!(stdout, SetAttribute(Attribute::Reverse))?;
    }
    write!(stdout, "{}", fit_to_width(text, width as usize))?;
    if selected {
        queue!(stdout, SetAttribute(Attribute::Reset))?;
    }
    Ok(())
}

fn fit_to_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(text) <= width {
        return text.to_string();
    }

    let ellipsis = "…";
    let max_width = width.saturating_sub(UnicodeWidthStr::width(ellipsis));
    let mut rendered = String::new();
    let mut used = 0;

    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_width > max_width {
            break;
        }
        rendered.push(ch);
        used += ch_width;
    }

    rendered.push('…');
    rendered
}

#[cfg(test)]
mod tests {
    use super::{fit_to_width, handle_key, TuiState};
    use crate::models::{sample_session, SearchResult};

    fn state() -> TuiState {
        TuiState {
            query: String::new(),
            selected_index: 0,
            limit: 8,
            status_message: String::new(),
        }
    }

    #[test]
    fn leaves_short_lines_unchanged() {
        assert_eq!(fit_to_width("ccx", 20), "ccx");
    }

    #[test]
    fn truncates_long_lines_with_ellipsis() {
        assert_eq!(fit_to_width("codex -- very long title", 12), "codex -- ve…");
    }

    #[test]
    fn q_can_start_a_query() {
        let mut state = state();
        let results = vec![SearchResult {
            session: sample_session(),
            score: 10,
            matched_terms: Vec::new(),
            score_breakdown: std::collections::BTreeMap::new(),
        }];

        let outcome = handle_key(
            &mut state,
            &results,
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('q'),
                crossterm::event::KeyModifiers::NONE,
            ),
        )
        .unwrap();

        assert!(outcome.is_none());
        assert_eq!(state.query, "q");
    }
}
