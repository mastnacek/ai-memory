use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use std::str::FromStr;
use uma_core::{domain::FactType, search::SearchMode, store::Store};

use crate::shared::scope::resolve_scope;

#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    /// Search query keywords or semantic concept
    #[arg(default_value = "")]
    pub query: String,

    /// Search mode: 'keyword' (BM25), 'semantic' (Vector), or 'hybrid' (RRF fusion)
    #[arg(short = 'm', long = "mode", default_value = "hybrid")]
    pub mode: String,

    /// Scope to search within (project name or "global"). Omit to search both.
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Filter by fact type (decision, preference, pattern, note, etc.)
    #[arg(short = 't', long = "type")]
    pub fact_type: Option<String>,

    /// Include deprecated/superseded facts in search results
    #[arg(long = "include-deprecated")]
    pub include_deprecated: bool,

    /// Point-in-time search: filter facts active at this ISO 8601 datetime (e.g. 2026-01-15T00:00:00Z)
    #[arg(long = "as-of")]
    pub as_of: Option<String>,

    /// Maximum number of search results to return
    #[arg(short = 'n', long = "limit", default_value = "10")]
    pub limit: usize,

    /// Force rebuilding the search index from markdown files before searching
    #[arg(long = "reindex")]
    pub reindex: bool,

    /// Vectorize all facts missing embedding vectors via OpenRouter
    #[arg(long = "vectorize")]
    pub vectorize: bool,
}

/// Executes the Search vertical slice: BM25, Semantic Vector, or Hybrid RRF search.
pub fn run(args: SearchArgs) -> Result<()> {
    if args.reindex {
        let count = Store::reindex_all()?;
        println!("Reindexed {} fact(s) into centralized index.", count);
    }

    if args.vectorize {
        println!("Vectorizing facts via OpenRouter embeddings...");
        match Store::vectorize_all() {
            Ok(count) => println!("Vectorized {} fact(s).", count),
            Err(e) => eprintln!("Vectorization warning: {}", e),
        }
    }

    if args.query.trim().is_empty() {
        if !args.reindex && !args.vectorize {
            println!("Please provide a search query: uma search <query>");
        }
        return Ok(());
    }

    let scope_filter = match args.scope {
        Some(ref s) => Some(resolve_scope(Some(s.clone()))?),
        None => None,
    };

    let type_filter = args
        .fact_type
        .as_deref()
        .map(FactType::from_str)
        .transpose()?;

    let mode = SearchMode::from_str(&args.mode).unwrap_or(SearchMode::Hybrid);
    let as_of_dt = args
        .as_of
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()?
        .map(|dt| dt.with_timezone(&Utc));

    let hits = Store::search_all(
        &args.query,
        scope_filter.as_ref(),
        type_filter.as_ref(),
        mode,
        args.include_deprecated,
        as_of_dt,
        args.limit,
    )?;

    if hits.is_empty() {
        println!(
            "No facts matching '{}' found (mode: {:?}).",
            args.query, mode
        );
        return Ok(());
    }

    println!(
        "Found {} matching fact(s) [Mode: {:?}]:\n",
        hits.len(),
        mode
    );
    for (idx, hit) in hits.iter().enumerate() {
        let status_badge = match hit.status {
            uma_core::domain::FactStatus::Stable => "",
            uma_core::domain::FactStatus::Deprecated => " [DEPRECATED]",
            uma_core::domain::FactStatus::Draft => " [DRAFT]",
        };

        println!(
            "{}. {} [{}] [{}]{} (score: {:.3})",
            idx + 1,
            hit.title,
            hit.scope,
            hit.fact_type,
            status_badge,
            hit.score
        );
        println!("   ID: {}", hit.id);
        if let Some(ref sup) = hit.supersedes {
            println!("   Supersedes: {}", sup);
        }
        if !hit.tags.is_empty() {
            println!("   Tags: {}", hit.tags.join(", "));
        }
        if !hit.snippet.is_empty() {
            println!("   Match: {}", hit.snippet);
        }
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(flatten)]
        args: SearchArgs,
    }

    #[test]
    fn test_search_args_parsing() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from([
            "test",
            "vertical slice",
            "--mode",
            "semantic",
            "--scope",
            "global",
            "--type",
            "decision",
            "--include-deprecated",
            "--limit",
            "5",
        ])?;

        assert_eq!(cli.args.query, "vertical slice");
        assert_eq!(cli.args.mode, "semantic");
        assert_eq!(cli.args.scope, Some("global".to_string()));
        assert_eq!(cli.args.fact_type, Some("decision".to_string()));
        assert!(cli.args.include_deprecated);
        assert_eq!(cli.args.limit, 5);
        assert!(!cli.args.reindex);
        assert!(!cli.args.vectorize);
        Ok(())
    }
}
