//! Recall: fastbrain-gated memory recall (the S3 compromise).
//!
//! **Read-only.** This slice answers one question — *is memory worth
//! searching for this prompt, and if so, which facts?* — and never mutates.
//! It exists because auto-injection was paused deliberately (operator
//! preference: uncontrolled injection caused context noise); the gate makes
//! injection *conditional* instead of absent.
//!
//! Flow: the judge (offline markers or Jev via OpenRouter) decides whether
//! the prompt may depend on remembered facts; only on a trigger does a BM25
//! keyword search run — never a network semantic call, because recall must
//! not add latency or cost to every turn. The caller decides what to do with
//! the hits; UMA itself injects nothing.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::str::FromStr as _;
use uma_core::domain::FactType;
use uma_core::fastbrain::{self, Judge};
use uma_core::search::SearchMode;
use uma_core::store::Store;

#[derive(Args, Debug, Clone)]
pub struct RecallArgs {
    #[command(subcommand)]
    pub command: RecallCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RecallCommand {
    /// Check a prompt for memory triggers and recall matching facts (read-only)
    Check(CheckArgs),
}

#[derive(Args, Debug, Clone)]
pub struct CheckArgs {
    /// The prompt to check (joined if several arguments are given)
    pub prompt: Vec<String>,

    /// Which judge to use: off (default) or jev
    #[arg(long = "judge", default_value = "off", value_parser = ["off", "jev"])]
    pub judge: String,

    /// Maximum facts to recall
    #[arg(long = "max", default_value_t = 3)]
    pub max: usize,

    /// Emit the verdict and recalled facts as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Recall vertical slice.
pub fn run(args: RecallArgs) -> Result<()> {
    match args.command {
        RecallCommand::Check(check) => check_prompt(check),
    }
}

fn judge_from_name(name: &str) -> Judge {
    match name {
        "jev" => Judge::Jev,
        _ => Judge::Offline,
    }
}

/// Recall verdict: trigger plus the facts worth showing.
#[derive(serde::Serialize)]
struct RecallVerdict {
    search: bool,
    judged_by: String,
    fact_types: Vec<String>,
    recalled: usize,
    facts: Vec<serde_json::Value>,
    note: Option<String>,
}

fn check_prompt(args: CheckArgs) -> Result<()> {
    if args.prompt.is_empty() {
        anyhow::bail!("Nothing to check: pass the prompt text");
    }
    let prompt = args.prompt.join(" ");

    // judge_recall_need already degrades Jev to offline with a note when the
    // transport fails, so one call covers both transports and their failure.
    let verdict = fastbrain::judge_recall_need(&prompt, judge_from_name(&args.judge))
        .map_err(|failure| anyhow::anyhow!("Recall check failed: {failure}"))?;
    let need = &verdict.answer;

    // No trigger means no search at all: the whole point of the gate.
    if !need.search {
        return emit(
            &RecallVerdict {
                search: false,
                judged_by: verdict.judged_by.as_str().to_string(),
                fact_types: Vec::new(),
                recalled: 0,
                facts: Vec::new(),
                note: verdict.notes,
            },
            args.json,
        );
    }

    let (facts, search_notes) = recall_facts(&prompt, &need.fact_types, args.max)?;
    // Merge provenance (judge note) with the search's degradation notes so a
    // relaxed recall is visible in the verdict, not silently presented as a
    // precise match.
    let note = match (verdict.notes.clone(), search_notes.is_empty()) {
        (Some(judge_note), true) => Some(judge_note),
        (Some(judge_note), false) => Some(format!("{judge_note}; {}", search_notes.join("; "))),
        (None, false) => Some(search_notes.join("; ")),
        (None, true) => None,
    };
    emit(
        &RecallVerdict {
            search: true,
            judged_by: verdict.judged_by.as_str().to_string(),
            fact_types: need.fact_types.clone(),
            recalled: facts.len(),
            facts,
            note,
        },
        args.json,
    )
}

fn emit(verdict: &RecallVerdict, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(verdict)?);
        return Ok(());
    }

    if !verdict.search {
        println!(
            "No memory trigger — recall skipped (judge {})",
            verdict.judged_by
        );
    } else {
        println!(
            "Recall trigger: {} (judge {}, {} fact(s))",
            if verdict.fact_types.is_empty() {
                "general".to_string()
            } else {
                verdict.fact_types.join(", ")
            },
            verdict.judged_by,
            verdict.recalled
        );
        for (idx, fact) in verdict.facts.iter().enumerate() {
            println!(
                "{}. [{}] {} — {}",
                idx + 1,
                fact["fact_type"].as_str().unwrap_or("?"),
                fact["title"].as_str().unwrap_or("?"),
                fact["id"].as_str().unwrap_or("?")
            );
        }
    }
    if let Some(ref note) = verdict.note {
        println!("note: {note}");
    }
    Ok(())
}

/// Recalls facts for the trigger: the prompt's meaningful tokens as a BM25
/// keyword query, filtered to the judge's subtypes when it named any.
fn recall_facts(
    prompt: &str,
    fact_types: &[String],
    max: usize,
) -> Result<(Vec<serde_json::Value>, Vec<String>)> {
    let query = recall_query(prompt);
    let subtypes: Vec<FactType> = fact_types
        .iter()
        .filter_map(|name| FactType::from_str(name).ok())
        .collect();

    let mut facts = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let filters: Vec<Option<FactType>> = if subtypes.is_empty() {
        vec![None]
    } else {
        subtypes
            .iter()
            .map(|fact_type| Some(fact_type.clone()))
            .collect()
    };
    for filter in filters {
        if facts.len() >= max {
            break;
        }
        let outcome = Store::search_all(
            &query,
            None,
            filter.as_ref(),
            SearchMode::Keyword,
            false,
            None,
            max - facts.len(),
        )?;
        // A relaxed (any-term) recall must be visible in the verdict, not
        // silently presented as a precise match.
        if let Some(note) = outcome.note() {
            if !notes.contains(&note) {
                notes.push(note);
            }
        }
        for hit in &outcome.hits {
            facts.push(serde_json::json!({
                "id": hit.id,
                "title": hit.title,
                "fact_type": hit.fact_type,
                "scope": hit.scope,
                "score": hit.score,
                "snippet": hit.snippet,
                "tags": hit.tags,
            }));
        }
    }
    facts.truncate(max);
    Ok((facts, notes))
}

/// Builds the search query from the prompt: meaningful words only, capped, so
/// BM25 keeps signal instead of matching everything vaguely.
fn recall_query(prompt: &str) -> String {
    const NOISE: &[&str] = &[
        "the", "and", "for", "with", "that", "this", "please", "could", "would", "does", "jak",
        "pro", "nebo", "když", "aby", "tedy",
    ];
    prompt
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.chars().count() > 2 && !NOISE.contains(&word.to_lowercase().as_str()))
        .take(12)
        .map(str::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests;
