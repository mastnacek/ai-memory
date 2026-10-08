//! Normalized reader for agent session stores.
//!
//! Two source formats are supported — pi agent and Claude Code — and both are
//! JSONL with the same essential shape: message lines carry a role and either a
//! plain string or a list of text blocks. Normalizing them here means the
//! session browser and the future import slice share one reader through the
//! kernel, rather than each slice re-implementing file walking.
//!
//! **Project identity comes from the session's own `cwd`, never from the
//! directory name.** The directory name is a lossy path encoding (separators
//! flattened), and the same project legitimately appears under several encoded
//! names — one per machine it was worked on.

mod lookup;
mod reader;
mod types;

pub use lookup::{default_roots, find_session, project_alive};
pub use reader::{read_detail, scan_at};
pub use types::{SessionDetail, SessionRecord, SessionSource};
