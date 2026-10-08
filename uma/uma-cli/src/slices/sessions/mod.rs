//! Session browser: read-only scan of the agent session stores.
//!
//! This is the browsing half of the import workflow. Importing is a separate
//! future slice, and the separation is deliberate: **browsing reads, only the
//! approval modal writes.** Keeping them apart means an operator can inspect
//! years of sessions without any fact being proposed, let alone stored.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use uma_core::sources::{
    default_roots, find_session, project_alive, read_detail, scan_at, SessionRecord,
};

#[derive(Args, Debug, Clone)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: SessionsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SessionsCommand {
    /// List sessions from the pi and Claude Code stores
    List(ListArgs),
    /// Show one session: counts plus its substantive user messages
    Show(ShowArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    /// Only sessions whose project path contains this text
    #[arg(long = "project", value_name = "TEXT")]
    pub project: Option<String>,

    /// Store to scan: pi, claude, or all (default)
    #[arg(long = "source", value_name = "PI|CLAUDE|ALL")]
    pub source: Option<String>,

    /// Sort order: new (default), old, or size
    #[arg(long = "sort", value_name = "NEW|OLD|SIZE")]
    pub sort: Option<String>,

    /// Maximum sessions to print (default 20; 0 = unlimited)
    #[arg(long = "limit", default_value_t = 20)]
    pub limit: usize,

    /// Emit as JSON
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ShowArgs {
    /// Session ID (prefix is enough) or a session file path
    pub session: String,

    /// Emit as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Sessions vertical slice.
///
/// Read-only. Project identity comes from the session's own `cwd`, never the
/// directory name — the encoded directory is lossy and machine-specific, while
/// the same project legitimately appears under several encoded names.
pub fn run(args: SessionsArgs) -> Result<()> {
    match args.command {
        SessionsCommand::List(list) => list_sessions(list),
        SessionsCommand::Show(show) => show_session(show),
    }
}

fn collect(source: Option<&str>) -> Result<Vec<SessionRecord>> {
    let wanted = source.unwrap_or("all").trim().to_lowercase();
    let mut records = Vec::new();
    for (source, root) in default_roots() {
        let include = match wanted.as_str() {
            "all" | "" => true,
            other => source.as_str() == other,
        };
        if !include {
            continue;
        }
        records.extend(scan_at(&root, source)?);
    }
    Ok(records)
}

fn list_sessions(args: ListArgs) -> Result<()> {
    let mut records = collect(args.source.as_deref())?;

    if let Some(project) = args.project.as_deref() {
        let needle = project.to_lowercase();
        records.retain(|record| record.project.to_lowercase().contains(&needle));
    }

    match args.sort.as_deref().unwrap_or("new") {
        "old" => records.reverse(),
        "size" => records.sort_by_key(|r| std::cmp::Reverse(r.size_bytes)),
        _ => {} // already newest first
    }

    if args.json {
        let payload: Vec<_> = records.iter().map(json_record).collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if records.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    let total = records.len();
    let shown = if args.limit == 0 {
        total
    } else {
        total.min(args.limit)
    };
    println!("{total} session(s), showing {shown} — [d] = project deleted from disk\n");
    println!(
        "  {:<6} {:<10} {:<2} {:>4}/{:<4} {:>8}  project · title",
        "source", "date", "d", "user", "subst", "size"
    );
    for record in &records[..shown] {
        println!("{}", line_for(record));
    }
    if args.project.is_some() {
        println!("\n(import: future `uma import sessions` proposes facts from these)");
    }
    Ok(())
}

fn line_for(record: &SessionRecord) -> String {
    let dead = if project_alive(record) { " " } else { "d" };
    let size = if record.size_bytes >= 1024 * 1024 {
        format!("{:.1}M", record.size_bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0}K", record.size_bytes as f64 / 1024.0)
    };
    let label = record
        .title
        .clone()
        .unwrap_or_else(|| record.session_id.chars().take(12).collect());
    format!(
        "  {:<6} {:<10} {:<2} {:>4}/{:<4} {:>8}  {}  · {}",
        record.source.as_str(),
        record.started_at.format("%Y-%m-%d"),
        dead,
        record.user_messages,
        record.substantive_user_turns,
        size,
        truncate(&record.project, 36),
        truncate(&label, 30)
    )
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let head: String = text.chars().take(width - 1).collect();
    format!("{head}…")
}

/// Secret patterns are counted and redacted, never reprinted: old sessions can
/// contain real keys, and the browser must be safe to scroll through.
const SECRET: &[(&str, &str)] = &[
    ("sk-", "sk-[REDACTED]"),
    ("ghp_", "ghp_[REDACTED]"),
    ("AKIA", "AKIA[REDACTED]"),
    ("xoxb-", "xox[REDACTED]"),
];

fn mask(text: &str) -> String {
    let mut masked = text.to_string();
    for (pattern, replacement) in SECRET {
        if let Some(start) = masked.find(pattern) {
            let tail = &masked[start..];
            let end = tail.find(char::is_whitespace).unwrap_or(tail.len());
            masked.replace_range(start..start + end, replacement);
        }
    }
    masked
}

fn json_record(record: &SessionRecord) -> serde_json::Value {
    serde_json::json!({
        "source": record.source.as_str(),
        "project": record.project,
        "session_id": record.session_id,
        "started_at": record.started_at.to_rfc3339(),
        "title": record.title,
        "size_bytes": record.size_bytes,
        "user_messages": record.user_messages,
        "assistant_messages": record.assistant_messages,
        "substantive_user_turns": record.substantive_user_turns,
        "project_alive": project_alive(record),
        "file": record.file.to_string_lossy(),
    })
}

fn show_session(args: ShowArgs) -> Result<()> {
    let Some((source, file)) = find_session(&args.session)? else {
        bail!(
            "No session matching '{}'. Use `uma sessions list` to find one.",
            args.session
        );
    };
    let detail = read_detail(&file, source)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "record": json_record(&detail.record),
                "assistant_messages": detail.assistant_messages,
                "tool_results": detail.tool_results,
                "user_messages": detail.user_messages,
            }))?
        );
        return Ok(());
    }

    let record = &detail.record;
    println!(
        "Session  {} ({})",
        record.session_id,
        record.source.as_str()
    );
    println!(
        "Project  {}{}",
        record.project,
        if project_alive(record) {
            ""
        } else {
            "  [DELETED from disk]"
        }
    );
    println!(
        "Started  {}   {} bytes   title: {}",
        record.started_at.format("%Y-%m-%d %H:%M UTC"),
        record.size_bytes,
        record.title.as_deref().unwrap_or("(none)")
    );
    println!(
        "Turns    {} user ({} substantive), {} assistant, {} tool result(s)",
        record.user_messages,
        record.substantive_user_turns,
        detail.assistant_messages,
        detail.tool_results
    );
    println!("File     {}", record.file.display());
    println!(
        "\nSubstantive user messages ({}):\n",
        detail.user_messages.len()
    );
    for message in &detail.user_messages {
        println!("  ▸ {}", mask(message));
    }
    Ok(())
}
