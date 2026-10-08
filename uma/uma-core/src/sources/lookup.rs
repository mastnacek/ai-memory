//! Locating session stores and sessions on this machine.

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::reader::scan_at;
use super::types::{SessionRecord, SessionSource};

/// The default roots for both stores on this machine.
///
/// Missing roots are simply not returned — a machine without Claude Code still
/// gets pi sessions, and neither absence should be an error.
pub fn default_roots() -> Vec<(SessionSource, PathBuf)> {
    let Some(base) = directories::BaseDirs::new() else {
        return Vec::new();
    };
    let home = base.home_dir().to_path_buf();
    vec![
        (
            SessionSource::PiAgent,
            home.join(".pi").join("agent").join("sessions"),
        ),
        (
            SessionSource::ClaudeCode,
            home.join(".claude").join("projects"),
        ),
    ]
}

/// True when a record's project directory still exists on disk.
pub fn project_alive(record: &SessionRecord) -> bool {
    Path::new(&record.project).is_dir()
}

/// Finds a session across the known roots by ID prefix, or accepts an explicit
/// file path. `Ok(None)` when nothing matches.
pub fn find_session(query: &str) -> Result<Option<(SessionSource, PathBuf)>> {
    let as_path = PathBuf::from(query);
    if as_path.is_file() {
        let source = if as_path.to_string_lossy().contains(".claude") {
            SessionSource::ClaudeCode
        } else {
            SessionSource::PiAgent
        };
        return Ok(Some((source, as_path)));
    }

    for (source, root) in default_roots() {
        for record in scan_at(&root, source)? {
            if record.session_id.starts_with(query) {
                return Ok(Some((source, record.file)));
            }
        }
    }
    Ok(None)
}
