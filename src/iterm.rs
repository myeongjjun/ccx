use std::collections::HashMap;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::classifier::normalize_session;
use crate::debug;
use crate::models::{RawSessionMetadata, SessionRecord};

#[derive(Debug, Clone, Deserialize)]
pub struct RawWindowState {
    pub window_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub miniaturized: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IsolateApplyReport {
    pub status: String,
    #[serde(default)]
    pub mismatches: Vec<String>,
    #[serde(default)]
    pub errors: Vec<IsolateWindowError>,
    #[serde(default)]
    pub windows: Vec<RawWindowState>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IsolateWindowError {
    pub window_id: String,
    pub phase: String,
    pub message: String,
}

pub fn build_collection_script() -> String {
    r#"tell application "iTerm"
  set output to "["
  set firstSession to true
  repeat with windowIndex from 1 to count of windows
    set windowRef to item windowIndex of windows
    set windowId to my safe_id(windowRef)
    set windowMiniaturized to my safe_miniaturized(windowRef)
    repeat with tabIndex from 1 to count of tabs of windowRef
      set tabRef to item tabIndex of tabs of windowRef
      set tabId to my safe_id(tabRef)
      repeat with sessionIndex from 1 to count of sessions of tabRef
        set sessionRef to item sessionIndex of sessions of tabRef
        if firstSession is false then
          set output to output & ","
        end if
        set firstSession to false
        set output to output & "{\"session_id\":\"" & my safe_id(sessionRef) & "\""
        set output to output & ",\"window_id\":\"" & windowId & "\""
        set output to output & ",\"tab_id\":\"" & tabId & "\""
        set output to output & ",\"miniaturized\":" & windowMiniaturized
        set output to output & ",\"title\":\"" & my safe_name(sessionRef) & "\""
        set output to output & ",\"badge\":\"" & my safe_badge(sessionRef) & "\""
        set output to output & ",\"tty\":\"" & my safe_tty(sessionRef) & "\""
        set output to output & ",\"foreground_command\":\"" & my safe_job_name(sessionRef) & "\""
        set output to output & "}"
      end repeat
    end repeat
  end repeat
  set output to output & "]"
  return output
end tell

on safe_id(object_ref)
  try
    return (id of object_ref) as string
  on error
    return ""
  end try
end safe_id

on safe_name(session_ref)
  try
    return my esc(name of session_ref as text)
  on error
    return ""
  end try
end safe_name

on safe_badge(session_ref)
  try
    return my esc((badge of session_ref) as text)
  on error
    return ""
  end try
end safe_badge

on safe_tty(session_ref)
  try
    return my esc((tty of session_ref) as text)
  on error
    return ""
  end try
end safe_tty

on safe_job_name(session_ref)
  return ""
end safe_job_name

on safe_miniaturized(window_ref)
  try
    if miniaturized of window_ref then
      return "true"
    end if
  on error
    return "false"
  end try
  return "false"
end safe_miniaturized

on esc(valueText)
  set valueText to my replace_text(return, " ", valueText)
  set valueText to my replace_text(linefeed, " ", valueText)
  set valueText to my replace_text(tab, " ", valueText)
  set valueText to my replace_text("\\", "\\\\", valueText)
  set valueText to my replace_text("\"", "\\\"", valueText)
  return valueText
end esc

on replace_text(find_text, replace_text, source_text)
  set AppleScript's text item delimiters to find_text
  set text_items to every text item of source_text
  set AppleScript's text item delimiters to replace_text
  set source_text to text_items as text
  set AppleScript's text item delimiters to ""
  return source_text
end replace_text
"#
    .to_string()
}

pub fn build_window_state_script() -> String {
    r#"tell application "iTerm"
  set output to "["
  set firstWindow to true
  repeat with windowIndex from 1 to count of windows
    set windowRef to item windowIndex of windows
    if firstWindow is false then
      set output to output & ","
    end if
    set firstWindow to false
    set output to output & "{\"window_id\":\"" & my safe_id(windowRef) & "\""
    set output to output & ",\"title\":\"" & my safe_window_title(windowRef) & "\""
    set output to output & ",\"miniaturized\":" & my safe_miniaturized(windowRef)
    set output to output & "}"
  end repeat
  set output to output & "]"
  return output
end tell

on safe_id(object_ref)
  try
    return (id of object_ref) as string
  on error
    return ""
  end try
end safe_id

on safe_window_title(window_ref)
  try
    return my esc(name of window_ref as text)
  on error
    return ""
  end try
end safe_window_title

on safe_miniaturized(window_ref)
  try
    if miniaturized of window_ref then
      return "true"
    end if
  on error
    return "false"
  end try
  return "false"
end safe_miniaturized

on esc(valueText)
  set valueText to my replace_text(return, " ", valueText)
  set valueText to my replace_text(linefeed, " ", valueText)
  set valueText to my replace_text(tab, " ", valueText)
  set valueText to my replace_text("\\", "\\\\", valueText)
  set valueText to my replace_text("\"", "\\\"", valueText)
  return valueText
end esc

on replace_text(find_text, replace_text, source_text)
  set AppleScript's text item delimiters to find_text
  set text_items to every text item of source_text
  set AppleScript's text item delimiters to replace_text
  set source_text to text_items as text
  set AppleScript's text item delimiters to ""
  return source_text
end replace_text
"#
    .to_string()
}

pub fn build_focus_script(session: &SessionRecord) -> String {
    build_select_script(session, true)
}

pub fn build_quiet_focus_script(session: &SessionRecord) -> String {
    build_select_script(session, false)
}

pub fn build_window_focus_script(window_id: &str) -> String {
    format!(
        r#"tell application "iTerm"
  activate
  repeat with windowIndex from 1 to count of windows
    set windowRef to item windowIndex of windows
    if my safe_id(windowRef) is "{window_id}" then
      set miniaturized of windowRef to false
      select windowRef
      return "ok"
    end if
  end repeat
end tell

return "window-not-found"

on safe_id(object_ref)
  try
    return (id of object_ref) as string
  on error
    return ""
  end try
end safe_id
"#,
        window_id = escape_applescript_string(window_id),
    )
}

pub fn build_isolate_script(keep_window_ids: &[String], focus_window_id: &str) -> String {
    let keep_ids = format_applescript_string_list(keep_window_ids);

    format!(
        r#"tell application "iTerm"
  set keepWindowIds to {keep_ids}
  set reportStatus to "window-not-found"
  set errorItems to {{}}
  activate
  set errorItems to my apply_isolate_state(keepWindowIds)
  delay 0.05
  set mismatchWindowIds to my find_isolate_mismatches(keepWindowIds)
  if (count of mismatchWindowIds) > 0 then
    set retryErrors to my apply_isolate_state(keepWindowIds)
    repeat with retryError in retryErrors
      copy retryError to end of errorItems
    end repeat
    delay 0.05
    set mismatchWindowIds to my find_isolate_mismatches(keepWindowIds)
  end if
  repeat with windowIndex from 1 to count of windows
    set windowRef to item windowIndex of windows
    set currentWindowId to ""
    try
      set currentWindowId to (id of windowRef) as string
    end try
    if currentWindowId is "{focus_window_id}" then
      try
        if miniaturized of windowRef then
          set miniaturized of windowRef to false
        end if
      end try
      try
        select windowRef
      end try
      if (count of mismatchWindowIds) > 0 then
        set reportStatus to "partial:" & my join_list(mismatchWindowIds, ",")
      else
        set reportStatus to "ok"
      end if
      exit repeat
    end if
  end repeat
end tell

return "{{\"status\":\"" & my esc(reportStatus) & "\",\"mismatches\":" & my json_string_list(mismatchWindowIds) & ",\"errors\":" & my json_object_list(errorItems) & ",\"windows\":" & my report_window_states() & "}}"

on apply_isolate_state(keepWindowIds)
  set errorItems to {{}}
  tell application "iTerm"
    set windowIds to my list_window_ids()
    repeat with currentWindowId in windowIds
      set windowRef to my find_window_by_id(currentWindowId as text)
      if windowRef is missing value then
        copy my make_error_item(currentWindowId as text, "apply", "window-not-found") to end of errorItems
      else
        try
          if keepWindowIds contains (currentWindowId as text) then
            if miniaturized of windowRef then
              set miniaturized of windowRef to false
            end if
          else
            if miniaturized of windowRef is false then
              set miniaturized of windowRef to true
            end if
          end if
        on error errorMessage
          copy my make_error_item(currentWindowId as text, "apply", errorMessage) to end of errorItems
        end try
      end if
    end repeat
  end tell
  return errorItems
end apply_isolate_state

on find_isolate_mismatches(keepWindowIds)
  set mismatchWindowIds to {{}}
  tell application "iTerm"
    repeat with windowIndex from 1 to count of windows
      set windowRef to item windowIndex of windows
      set currentWindowId to ""
      try
        set currentWindowId to (id of windowRef) as string
      end try
      try
        if keepWindowIds contains currentWindowId then
          if miniaturized of windowRef then
            copy currentWindowId to end of mismatchWindowIds
          end if
        else
          if miniaturized of windowRef is false then
            copy currentWindowId to end of mismatchWindowIds
          end if
        end if
      on error
        copy currentWindowId to end of mismatchWindowIds
      end try
    end repeat
  end tell
  return mismatchWindowIds
end find_isolate_mismatches

on join_list(values, delimiter)
  if (count of values) is 0 then
    return ""
  end if
  set oldDelimiters to AppleScript's text item delimiters
  set AppleScript's text item delimiters to delimiter
  set joined to values as text
  set AppleScript's text item delimiters to oldDelimiters
  return joined
end join_list

on json_string_list(values)
  if (count of values) is 0 then
    return "[]"
  end if

  set jsonItems to {{}}
  repeat with itemValue in values
    copy "\"" & my esc(itemValue as text) & "\"" to end of jsonItems
  end repeat

  return "[" & my join_list(jsonItems, ",") & "]"
end json_string_list

on json_object_list(values)
  if (count of values) is 0 then
    return "[]"
  end if

  return "[" & my join_list(values, ",") & "]"
end json_object_list

on make_error_item(windowId, phaseName, errorMessage)
  return "{{\"window_id\":\"" & my esc(windowId) & "\",\"phase\":\"" & my esc(phaseName) & "\",\"message\":\"" & my esc(errorMessage) & "\"}}"
end make_error_item

on report_window_states()
  set jsonItems to {{}}
  tell application "iTerm"
    repeat with windowIndex from 1 to count of windows
      set windowRef to item windowIndex of windows
      set currentWindowId to ""
      set currentMiniaturized to "false"
      try
        set currentWindowId to (id of windowRef) as string
      end try
      try
        if miniaturized of windowRef then
          set currentMiniaturized to "true"
        end if
      end try
      copy "{{\"window_id\":\"" & my esc(currentWindowId) & "\",\"title\":\"\",\"miniaturized\":" & currentMiniaturized & "}}" to end of jsonItems
    end repeat
  end tell

  return "[" & my join_list(jsonItems, ",") & "]"
end report_window_states

on list_window_ids()
  set windowIds to {{}}
  tell application "iTerm"
    repeat with windowRef in windows
      try
        copy ((id of windowRef) as string) to end of windowIds
      end try
    end repeat
  end tell
  return windowIds
end list_window_ids

on find_window_by_id(targetWindowId)
  tell application "iTerm"
    repeat with windowRef in windows
      try
        if ((id of windowRef) as string) is targetWindowId then
          return windowRef
        end if
      end try
    end repeat
  end tell
  return missing value
end find_window_by_id

on esc(valueText)
  set valueText to my replace_text(return, " ", valueText)
  set valueText to my replace_text(linefeed, " ", valueText)
  set valueText to my replace_text(tab, " ", valueText)
  set valueText to my replace_text("\\", "\\\\", valueText)
  set valueText to my replace_text("\"", "\\\"", valueText)
  return valueText
end esc

on replace_text(find_text, replace_text, source_text)
  set AppleScript's text item delimiters to find_text
  set text_items to every text item of source_text
  set AppleScript's text item delimiters to replace_text
  set source_text to text_items as text
  set AppleScript's text item delimiters to ""
  return source_text
end replace_text

"#,
        keep_ids = keep_ids,
        focus_window_id = escape_applescript_string(focus_window_id),
    )
}

pub fn build_session_contents_script(session_id: &str) -> String {
    format!(
        r#"tell application "iTerm"
  repeat with windowRef in windows
    repeat with tabRef in tabs of windowRef
      repeat with sessionRef in sessions of tabRef
        if my safe_id(sessionRef) is "{session_id}" then
          return contents of sessionRef
        end if
      end repeat
    end repeat
  end repeat
end tell

return ""

on safe_id(object_ref)
  try
    return (id of object_ref) as string
  on error
    return ""
  end try
end safe_id
"#,
        session_id = escape_applescript_string(session_id),
    )
}

fn build_select_script(session: &SessionRecord, activate_window: bool) -> String {
    let activate_line = if activate_window { "  activate\n" } else { "" };
    let window_select_line = if activate_window {
        "      select windowRef\n"
    } else {
        ""
    };

    format!(
        r#"tell application "iTerm"
{activate}  repeat with windowIndex from 1 to count of windows
    set windowRef to item windowIndex of windows
    if my safe_id(windowRef) is "{window_id}" then
      repeat with tabIndex from 1 to count of tabs of windowRef
        set tabRef to item tabIndex of tabs of windowRef
        repeat with sessionIndex from 1 to count of sessions of tabRef
          set sessionRef to item sessionIndex of sessions of tabRef
          if my safe_id(sessionRef) is "{session_id}" then
{window_select}            select tabRef
            select sessionRef
            return "ok"
          end if
        end repeat
      end repeat
    end if
  end repeat
end tell

return "session-not-found"

on safe_id(object_ref)
  try
    return (id of object_ref) as string
  on error
    return ""
  end try
end safe_id
"#,
        activate = activate_line,
        window_select = window_select_line,
        window_id = escape_applescript_string(&session.window_id),
        session_id = escape_applescript_string(&session.session_id),
    )
}

pub fn collect_raw_sessions() -> Result<Vec<RawSessionMetadata>> {
    let script = build_collection_script();
    debug::log("collecting raw sessions from iTerm");
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("failed to invoke osascript for collection")?;

    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    let payload = String::from_utf8_lossy(&output.stdout);
    let sessions = serde_json::from_str::<Vec<RawSessionMetadata>>(&payload)
        .context("failed to parse collected iTerm2 session JSON")?;
    debug::log(format!("collected {} raw sessions", sessions.len()));
    Ok(sessions)
}

pub fn collect_window_states() -> Result<Vec<RawWindowState>> {
    let script = build_window_state_script();
    debug::log("collecting live window states from iTerm");
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("failed to invoke osascript for live window state collection")?;

    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    let payload = String::from_utf8_lossy(&output.stdout);
    let windows = serde_json::from_str::<Vec<RawWindowState>>(&payload)
        .context("failed to parse collected iTerm2 window state JSON")?;
    debug::log(format!("collected {} live window states", windows.len()));
    Ok(windows)
}

pub fn collect_sessions() -> Result<Vec<SessionRecord>> {
    let mut sessions = collect_raw_sessions()?;
    enrich_raw_sessions_from_tty(&mut sessions);
    let normalized: Vec<_> = sessions.into_iter().map(normalize_session).collect();
    debug::log(format!("normalized {} sessions", normalized.len()));
    Ok(normalized)
}

pub fn focus_session(session: &SessionRecord) -> Result<String> {
    let script = build_focus_script(session);
    run_script(&script)
}

pub fn focus_session_async(session: &SessionRecord) -> Result<()> {
    let script = build_focus_script(session);
    run_script_async(&script)
}

pub fn focus_window(window_id: &str) -> Result<String> {
    let script = build_window_focus_script(window_id);
    run_script(&script)
}

pub fn focus_window_async(window_id: &str) -> Result<()> {
    let script = build_window_focus_script(window_id);
    run_script_async(&script)
}

pub fn quietly_focus_session(session: &SessionRecord) -> Result<String> {
    let script = build_quiet_focus_script(session);
    run_script(&script)
}

pub fn quietly_focus_session_async(session: &SessionRecord) -> Result<()> {
    let script = build_quiet_focus_script(session);
    run_script_async(&script)
}

pub fn isolate_windows_async(keep_window_ids: &[String], focus_window_id: &str) -> Result<()> {
    let script = build_isolate_script(keep_window_ids, focus_window_id);
    run_script_async(&script)
}

pub fn isolate_windows_report(
    keep_window_ids: &[String],
    focus_window_id: &str,
) -> Result<IsolateApplyReport> {
    debug::log(format!(
        "applying isolate keep_window_ids={keep_window_ids:?} focus_window_id={focus_window_id}"
    ));
    let script = build_isolate_script(keep_window_ids, focus_window_id);
    let payload = run_script(&script)?;
    serde_json::from_str(&payload).context("failed to parse isolate apply report JSON")
}

pub fn session_contents(session_id: &str) -> Result<String> {
    let script = build_session_contents_script(session_id);
    run_script(&script)
}

fn run_script(script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("failed to invoke osascript")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim())
    }
}

fn run_script_async(script: &str) -> Result<()> {
    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn osascript")?;
    Ok(())
}

fn escape_applescript_string(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

fn format_applescript_string_list(values: &[String]) -> String {
    if values.is_empty() {
        "{}".to_string()
    } else {
        let joined = values
            .iter()
            .map(|value| format!("\"{}\"", escape_applescript_string(value)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{joined}}}")
    }
}

#[derive(Debug, Clone)]
struct TtyMetadata {
    cwd: Option<String>,
    foreground_command: Option<String>,
    process_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TtyProcess {
    pid: u32,
    ppid: u32,
    stat: String,
    comm: String,
}

fn enrich_raw_sessions_from_tty(sessions: &mut [RawSessionMetadata]) {
    let mut tty_cache: HashMap<String, Option<TtyMetadata>> = HashMap::new();

    for session in sessions {
        if session.cwd.is_some() && session.foreground_command.is_some() {
            continue;
        }

        let Some(tty_name) = session
            .tty
            .as_deref()
            .and_then(normalize_tty_name)
            .map(str::to_string)
        else {
            continue;
        };

        let metadata = tty_cache
            .entry(tty_name.clone())
            .or_insert_with(|| inspect_tty_metadata(&tty_name).ok())
            .clone();

        let Some(metadata) = metadata else {
            continue;
        };

        if session.cwd.is_none() {
            session.cwd = metadata.cwd.clone();
        }
        if session.foreground_command.is_none() {
            session.foreground_command = metadata.foreground_command.clone();
        }
        if session.process_name.is_none() {
            session.process_name = metadata.process_name.clone();
        }
    }
}

fn inspect_tty_metadata(tty_name: &str) -> Result<TtyMetadata> {
    let processes = collect_tty_processes(tty_name)?;
    let selected = select_tty_process(&processes).context("no tty process candidates")?;
    let cwd = lookup_process_cwd(selected.pid).ok();
    let foreground_command = Some(selected.comm.clone());
    let process_name = Some(selected.comm.clone());

    Ok(TtyMetadata {
        cwd,
        foreground_command,
        process_name,
    })
}

fn collect_tty_processes(tty_name: &str) -> Result<Vec<TtyProcess>> {
    let output = Command::new("ps")
        .args(["-t", tty_name, "-o", "pid=,ppid=,stat=,comm="])
        .output()
        .with_context(|| format!("failed to inspect tty processes for {tty_name}"))?;

    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let processes = stdout
        .lines()
        .filter_map(parse_ps_process_line)
        .collect::<Vec<_>>();

    Ok(processes)
}

fn parse_ps_process_line(line: &str) -> Option<TtyProcess> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let pid = parts.next()?.parse().ok()?;
    let ppid = parts.next()?.parse().ok()?;
    let stat = parts.next()?.to_string();
    let comm = parts.next()?.to_string();

    Some(TtyProcess {
        pid,
        ppid,
        stat,
        comm,
    })
}

fn select_tty_process(processes: &[TtyProcess]) -> Option<&TtyProcess> {
    processes
        .iter()
        .find(|process| process.stat.contains('+') && !is_shell_command(&process.comm))
        .or_else(|| processes.iter().find(|process| process.stat.contains('+')))
        .or_else(|| {
            processes
                .iter()
                .rev()
                .find(|process| !is_shell_command(&process.comm))
        })
        .or_else(|| processes.last())
}

fn is_shell_command(command: &str) -> bool {
    matches!(
        command,
        "zsh" | "-zsh" | "bash" | "-bash" | "sh" | "-sh" | "fish" | "-fish"
    )
}

fn lookup_process_cwd(pid: u32) -> Result<String> {
    let output = Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .with_context(|| format!("failed to inspect cwd for pid {pid}"))?;

    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_lsof_cwd_output(&stdout).context("cwd entry missing from lsof output")
}

fn parse_lsof_cwd_output(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix('n'))
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
}

fn normalize_tty_name(raw_tty: &str) -> Option<&str> {
    let trimmed = raw_tty.trim();
    if trimmed.is_empty() {
        None
    } else if let Some(stripped) = trimmed.strip_prefix("/dev/") {
        Some(stripped)
    } else {
        Some(trimmed)
    }
}

#[cfg(test)]
mod process_tests {
    use super::{
        normalize_tty_name, parse_lsof_cwd_output, parse_ps_process_line, select_tty_process,
        TtyProcess,
    };

    #[test]
    fn parses_ps_process_line() {
        let process = parse_ps_process_line("123 1 S+ codex").expect("process");
        assert_eq!(process.pid, 123);
        assert_eq!(process.ppid, 1);
        assert_eq!(process.stat, "S+");
        assert_eq!(process.comm, "codex");
    }

    #[test]
    fn parses_lsof_cwd_output() {
        let output = "p123\nfcwd\nn/Users/kakao/personal/ccx\n";
        assert_eq!(
            parse_lsof_cwd_output(output).as_deref(),
            Some("/Users/kakao/personal/ccx")
        );
    }

    #[test]
    fn normalizes_dev_tty_prefix() {
        assert_eq!(normalize_tty_name("/dev/ttys001"), Some("ttys001"));
        assert_eq!(normalize_tty_name("ttys002"), Some("ttys002"));
    }

    #[test]
    fn prefers_foreground_non_shell_process() {
        let processes = vec![
            TtyProcess {
                pid: 1,
                ppid: 0,
                stat: "Ss".into(),
                comm: "zsh".into(),
            },
            TtyProcess {
                pid: 2,
                ppid: 1,
                stat: "S+".into(),
                comm: "codex".into(),
            },
        ];

        let selected = select_tty_process(&processes).expect("selected");
        assert_eq!(selected.comm, "codex");
    }
}
