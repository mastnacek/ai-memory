//! Scanning and reading session files across both store formats.

use std::io::BufRead;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use walkdir::WalkDir;

use super::types::{SessionDetail, SessionRecord, SessionSource};

/// Extracts readable text from a message content field, which may be a plain
/// string or a list of typed blocks (`{"type":"text","text":"..."}`).
fn content_text(content: &Value) -> Option<String> {
    match content {
        Value::String(text) => Some(text.clone()),
        Value::Array(blocks) => {
            let mut text = String::new();
            for block in blocks {
                if block.get("type").and_then(Value::as_str) == Some("text") {
                    if let Some(part) = block.get("text").and_then(Value::as_str) {
                        text.push_str(part);
                    }
                }
            }
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}

/// True for injected/system-looking user turns (tool caveats, command wrappers).
fn is_substantive(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('<')
        && !trimmed.starts_with("[Request interrupted")
}

/// One message line, normalized across both formats.
///
/// Both formats nest the payload under `message`; pi's `type` is always
/// `"message"` while Claude Code's equals the role.
struct Line<'a> {
    role: &'a str,
    text: Option<String>,
}

fn parse_line(obj: &Value) -> Option<Line<'_>> {
    let kind = obj.get("type")?.as_str()?;
    let message = obj.get("message")?;
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .or_else(|| match kind {
            "user" | "assistant" | "toolResult" => Some(kind),
            _ => None,
        })?;
    let text = content_text(message.get("content")?);
    Some(Line { role, text })
}

/// Scans every `*.jsonl` session under `root`, one record per file, newest
/// first. Session files sit one directory deep (`<root>/<project>/*.jsonl`);
/// the walk is bounded to that shape so stray files elsewhere are ignored.
///
/// Each file is streamed line by line, so a 21 MB session is read without ever
/// being held in memory. A file with unparsable lines still yields a record
/// when its header was readable - an old or truncated session should be
/// *listed*, not silently dropped.
pub fn scan_at(root: &Path, source: SessionSource) -> Result<Vec<SessionRecord>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    for project_dir in WalkDir::new(root).max_depth(1).min_depth(1).into_iter().flatten() {
        if !project_dir.file_type().is_dir() {
            continue;
        }
        for entry in WalkDir::new(project_dir.path()).max_depth(1).into_iter().flatten() {
            let path = entry.path();
            if !entry.file_type().is_file() || path.extension().is_some_and(|x| x != "jsonl") {
                continue;
            }
            if let Some(record) = read_session(path, source)? {
                records.push(record);
            }
        }
    }
    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(records)
}

/// Reads one session file into a record, streaming line by line.
fn read_session(path: &Path, source: SessionSource) -> Result<Option<SessionRecord>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open session {}", path.display()))?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);

    let mut cwd: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut started_at: Option<DateTime<Utc>> = None;
    let mut title: Option<String> = None;
    let mut user_messages = 0usize;
    let mut assistant_messages = 0usize;
    let mut substantive = 0usize;

    for line in std::io::BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        let Ok(obj) = serde_json::from_str::<Value>(&line) else { continue };

        // pi's header line carries id/timestamp; Claude Code stamps every
        // line and names the file after the session UUID.
        if cwd.is_none() {
            cwd = obj.get("cwd").and_then(Value::as_str).map(String::from);
        }
        if session_id.is_none() {
            session_id = obj
                .get("sessionId")
                .and_then(Value::as_str)
                .or_else(|| obj.get("id").and_then(Value::as_str))
                .map(String::from);
        }
        if started_at.is_none() {
            started_at = first_timestamp(&obj);
        }
        if title.is_none() {
            title = obj.get("aiTitle").and_then(Value::as_str).map(String::from);
        }

        let Some(line) = parse_line(&obj) else { continue };
        match line.role {
            "user" => {
                user_messages += 1;
                if line.text.as_deref().map(is_substantive).unwrap_or(false) {
                    substantive += 1;
                }
            }
            "assistant" => assistant_messages += 1,
            _ => {}
        }
    }

    let Some(session_id) = session_id else { return Ok(None) };
    Ok(Some(SessionRecord {
        source,
        project: cwd.unwrap_or_else(|| "(unknown)".to_string()),
        session_id,
        started_at: started_at.unwrap_or_default(),
        title,
        file: path.to_path_buf(),
        size_bytes: size,
        user_messages,
        assistant_messages,
        substantive_user_turns: substantive,
    }))
}

fn first_timestamp(obj: &Value) -> Option<DateTime<Utc>> {
    obj.get("timestamp")
        .and_then(Value::as_str)
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

/// Reads one session in full: record plus the substantive user turns, for
/// reviewing a session before any of it is proposed as memory.
pub fn read_detail(path: &Path, source: SessionSource) -> Result<SessionDetail> {
    let record = read_session(path, source)?.context("session file could not be read")?;

    let file = std::fs::File::open(path)?;
    let mut user_messages = Vec::new();
    let mut assistant_messages = 0usize;
    let mut tool_results = 0usize;

    for line in std::io::BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        let Ok(obj) = serde_json::from_str::<Value>(&line) else { continue };
        let Some(line) = parse_line(&obj) else { continue };
        // pi marks tool results by role (type stays "message"); Claude Code
        // emits them as user turns with tool_result blocks.
        if line.role == "assistant" {
            assistant_messages += 1;
        }
        if line.role == "toolResult" {
            tool_results += 1;
        }
        if line.role != "user" {
            continue;
        }
        let Some(text) = line.text else { continue };
        if !is_substantive(&text) {
            continue;
        }
        let mut text = text.trim().to_string();
        if text.len() > 500 {
            text.truncate(500);
            text.push_str("...");
        }
        user_messages.push(text);
    }

    Ok(SessionDetail {
        record,
        user_messages,
        assistant_messages,
        tool_results,
    })
}

#[cfg(test)]
mod tests;