//! Session record types shared by the browser and the import slice.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Which session store a record came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SessionSource {
    PiAgent,
    ClaudeCode,
}

impl SessionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionSource::PiAgent => "pi",
            SessionSource::ClaudeCode => "claude",
        }
    }
}

/// A session, summarized for listing and browsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionRecord {
    pub source: SessionSource,
    /// Project identity, decoded from the session's own `cwd` field.
    pub project: String,
    /// The session's own identifier (ULID for pi, UUID for Claude Code).
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    /// Session title, when the store records one (Claude Code `ai-title`).
    pub title: Option<String>,
    pub file: PathBuf,
    pub size_bytes: u64,
    pub user_messages: usize,
    pub assistant_messages: usize,
    /// Substantive user turns: not empty, not system-injected markup.
    pub substantive_user_turns: usize,
}

/// The full content of one session, for reading it before importing anything.
#[derive(Debug, Clone, Serialize)]
pub struct SessionDetail {
    pub record: SessionRecord,
    /// Substantive user messages, in order, truncated to a reviewable length.
    pub user_messages: Vec<String>,
    pub assistant_messages: usize,
    pub tool_results: usize,
}