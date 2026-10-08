//! Session import: turn agent-session history into memory candidates.
//!
//! **This slice proposes; it never writes.** The store is mutated only through
//! the gated `uma_write`/`uma_supersede` tools, where the operator sees exactly
//! what is being saved. That is the same contract as the consolidator, applied
//! to a source that can contain secrets and long-obsolete claims: a bulk import
//! that wrote directly would be a silent rewrite of memory with no consent.

use anyhow::Result;
use clap::{Args, Subcommand};
use uma_core::sources::{
    candidates_from_many, find_session, read_detail, scan_at, Candidate, SessionDetail,
};

#[derive(Args, Debug, Clone)]
pub struct ImportArgs {
    #[command(subcommand)]
    pub command: ImportCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ImportCommand {
    /// Extract memory candidates from agent sessions (read-only report)
    Sessions(ImportSessionsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ImportSessionsArgs {
    /// Only sessions whose project path contains this text
    #[arg(long = "project", value_name = "TEXT")]
    pub project: Option<String>,

    /// Store to scan: pi, claude, or all (default)
    #[arg(long = "source", value_name = "PI|CLAUDE|ALL")]
    pub source: Option<String>,

    /// One specific session (ID prefix or file path) instead of a scan
    #[arg(long = "session", value_name = "ID|PATH")]
    pub session: Option<String>,

    /// How many sessions to read at most (default 10, most recent first)
    #[arg(long = "limit", default_value_t = 10)]
    pub limit: usize,

    /// Maximum candidates one session may contribute (default 3)
    #[arg(long = "max-per-session", default_value_t = 3)]
    pub max_per_session: usize,

    /// Emit the candidate report as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Import vertical slice.
pub fn run(args: ImportArgs) -> Result<()> {
    match args.command {
        ImportCommand::Sessions(sessions) => import_sessions(sessions),
    }
}

fn import_sessions(args: ImportSessionsArgs) -> Result<()> {
    let details = collect(&args)?;
    let candidates = candidates_from_many(&details, args.max_per_session);

    if args.json {
        let payload: Vec<_> = candidates
            .iter()
            .map(|candidate| {
                serde_json::json!({
                    "kind": candidate.kind.as_str(),
                    "suggested_title": candidate.suggested_title,
                    "quote": candidate.quote,
                    "session_id": candidate.session_id,
                    "project": candidate.project,
                    "command": write_command_for(candidate),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!(
        "{} session(s) read, {} candidate(s) found — nothing written\n",
        details.len(),
        candidates.len()
    );

    if candidates.is_empty() {
        println!("No candidates. Broaden --project / --limit, or raise --max-per-session.");
        return Ok(());
    }

    for (idx, candidate) in candidates.iter().enumerate() {
        println!(
            "{}. [{}] {}",
            idx + 1,
            candidate.kind.as_str(),
            candidate.suggested_title
        );
        println!("   from: {} · {}", candidate.session_id, candidate.project);
        println!("   quote: {}", truncate(&candidate.quote, 140));
        println!("   propose with: {}", write_command_for(candidate));
        println!();
    }

    println!(
        "Review these, then propose the worthwhile ones through `uma_write` — that is the gated \
         path, and the only one that writes. Titles and bodies are suggestions: rewrite them \
         (and translate) before proposing."
    );
    Ok(())
}

/// Reads the sessions the arguments select, most recent first.
fn collect(args: &ImportSessionsArgs) -> Result<Vec<SessionDetail>> {
    let details: Vec<SessionDetail> = match args.session.as_deref() {
        Some(query) => {
            let Some((source, file)) = find_session(query)? else {
                anyhow::bail!("No session matching '{query}'. Find one with `uma sessions list`.");
            };
            vec![read_detail(&file, source)?]
        }
        None => {
            let wanted = args.source.as_deref().unwrap_or("all").trim().to_lowercase();
            let mut records = Vec::new();
            for (source, root) in uma_core::sources::default_roots() {
                let include = match wanted.as_str() {
                    "all" | "" => true,
                    other => source.as_str() == other,
                };
                if include {
                    records.extend(scan_at(&root, source)?);
                }
            }
            if let Some(project) = args.project.as_deref() {
                let needle = project.to_lowercase();
                records.retain(|record| record.project.to_lowercase().contains(&needle));
            }
            records.truncate(args.limit);
            let mut details = Vec::new();
            for record in &records {
                details.push(read_detail(&record.file, record.source)?);
            }
            details
        }
    };

    Ok(details)
}

/// The exact gated command that would propose this candidate.
///
/// `--since` keeps the session's date as the fact's start of validity, so an
/// imported decision does not masquerade as being made today.
fn write_command_for(candidate: &Candidate) -> String {
    format!(
        "uma write --type {} --title \"{}\" --body \"<reviewed body>\" --since <session date>",
        candidate.kind.as_str(),
        candidate.suggested_title.replace('"', "'")
    )
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let head: String = text.chars().take(width - 1).collect();
    format!("{head}…")
}

#[cfg(test)]
mod tests;