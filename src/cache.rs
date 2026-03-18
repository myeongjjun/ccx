use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};

use crate::models::SessionRecord;

pub const DEFAULT_CACHE_STALE_AFTER: Duration = Duration::from_secs(15);

pub fn default_cache_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        if let Some(home) = home_dir() {
            return home.join("Library/Caches/ccx");
        }
    }

    if let Ok(xdg_cache_home) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg_cache_home).join("ccx");
    }

    if let Some(home) = home_dir() {
        return home.join(".cache/ccx");
    }

    PathBuf::from(".ccx")
}

pub fn default_cache_path() -> PathBuf {
    default_cache_dir().join("latest-sessions.json")
}

pub fn cache_exists(path: &Path) -> bool {
    path.exists()
}

pub fn cache_is_stale(path: &Path, max_age: Duration) -> Result<bool> {
    let modified_at = fs::metadata(path)
        .with_context(|| format!("unable to stat {}", path.display()))?
        .modified()
        .with_context(|| format!("unable to read modified time for {}", path.display()))?;
    let age = SystemTime::now()
        .duration_since(modified_at)
        .unwrap_or_default();
    Ok(age > max_age)
}

pub fn load_sessions(path: &Path) -> Result<Vec<SessionRecord>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("unable to read {}", path.display()))?;
    let sessions = serde_json::from_str::<Vec<SessionRecord>>(&text)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    Ok(sessions)
}

pub fn load_single_session(path: &Path) -> Result<SessionRecord> {
    let text =
        fs::read_to_string(path).with_context(|| format!("unable to read {}", path.display()))?;
    let session = serde_json::from_str::<SessionRecord>(&text)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    Ok(session)
}

pub fn write_sessions(path: &Path, sessions: &[SessionRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("unable to create {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(sessions)?;
    let tmp_path = temporary_path(path);
    fs::write(&tmp_path, format!("{text}\n"))
        .with_context(|| format!("unable to write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "unable to atomically replace {} with {}",
            path.display(),
            tmp_path.display()
        )
    })?;
    Ok(())
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut file_name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_else(|| "latest-sessions.json".into());
    file_name.push(format!(".tmp-{}", std::process::id()));
    path.with_file_name(file_name)
}
