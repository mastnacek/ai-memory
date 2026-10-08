use std::str::FromStr;

use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use uma_core::domain::{FactId, FactType};
use uma_core::timeline::{build_chains, chain_for, Chain};

use crate::shared::{scope::resolve_scope, store_helper::get_store};

#[derive(Args, Debug, Clone)]
pub struct TimelineArgs {
    /// Show only the supersession chain containing this fact ID (ULID)
    #[arg(long = "id")]
    pub id: Option<String>,

    /// Scope (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Only consider facts of this type
    #[arg(short = 't', long = "type")]
    pub fact_type: Option<String>,

    /// Include chains with a single revision (facts never superseded)
    #[arg(long = "all")]
    pub all: bool,

    /// Emit as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Timeline vertical slice: reconstructs supersession history.
///
/// Strictly read-only. Deprecated facts are always loaded, because they *are* the
/// history — filtering them out would leave every chain with one visible step.
pub fn run(args: TimelineArgs) -> Result<()> {
    let scope = resolve_scope(args.scope)?;
    let fact_type = args
        .fact_type
        .as_deref()
        .map(FactType::from_str)
        .transpose()?;
    let store = get_store(&scope)?;
    let facts = store.list(&scope, fact_type.as_ref())?;

    let chains: Vec<Chain> = match args.id.as_deref() {
        Some(raw) => {
            let id = FactId::from_str(raw)?;
            // `chain_for` includes singletons so a factless-of-history id still
            // resolves, and we can say so explicitly instead of printing nothing.
            match chain_for(&facts, &id) {
                Some(chain) => vec![chain],
                None => bail!("No fact matching '{raw}' found in {scope}."),
            }
        }
        None => build_chains(&facts, args.all),
    };

    if args.json {
        let payload: Vec<_> = chains.iter().map(chain_to_json).collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let with_history = chains.iter().filter(|c| c.has_history()).count();
    if chains.is_empty() {
        println!("No supersession history in {scope}.");
        println!("Nothing has been superseded yet — nothing is modified by this command.");
        return Ok(());
    }

    println!(
        "Timeline — {with_history} chain(s) with history, {} shown, in {scope}\n",
        chains.len()
    );

    for (idx, chain) in chains.iter().enumerate() {
        let tip_title = chain.tip().map(|s| s.title.as_str()).unwrap_or("(empty)");
        println!(
            "{}. {} — {} revision(s)",
            idx + 1,
            tip_title,
            chain.steps.len()
        );

        for (step_idx, step) in chain.steps.iter().enumerate() {
            let marker = if step.status == uma_core::domain::FactStatus::Stable {
                "← current"
            } else {
                ""
            };
            println!(
                "   {}. [{}] {}  {}   {} {}",
                step_idx + 1,
                step.status,
                step.id,
                step.title,
                step.revised_at.format("%Y-%m-%d %H:%M"),
                marker
            );
        }
        println!();
    }

    Ok(())
}

fn chain_to_json(chain: &Chain) -> serde_json::Value {
    json!({
        "length": chain.steps.len(),
        "has_history": chain.has_history(),
        "current": chain.active().map(|s| s.id.to_string()),
        "revisions": chain.steps.iter().map(|step| json!({
            "id": step.id.to_string(),
            "title": step.title,
            "status": step.status.to_string(),
            "since": step.since.to_rfc3339(),
            "revised_at": step.revised_at.to_rfc3339(),
            "until": step.until.map(|u| u.to_rfc3339()),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    #[command(name = "uma")]
    struct TestCli {
        #[command(subcommand)]
        command: Top,
    }

    #[derive(clap::Subcommand, Debug)]
    enum Top {
        Timeline(TimelineArgs),
    }

    fn args(argv: &[&str]) -> TimelineArgs {
        match TestCli::try_parse_from(argv).expect("should parse").command {
            Top::Timeline(a) => a,
        }
    }

    #[test]
    fn test_timeline_defaults() {
        let a = args(&["uma", "timeline"]);
        assert!(a.id.is_none());
        assert!(!a.all, "single-revision facts stay hidden by default");
        assert!(!a.json);
    }

    #[test]
    fn test_timeline_accepts_id_and_all() {
        let a = args(&[
            "uma",
            "timeline",
            "--id",
            "01M4D6K8J6QGDFC0Y11FW45R87",
            "--all",
        ]);
        assert_eq!(a.id.as_deref(), Some("01M4D6K8J6QGDFC0Y11FW45R87"));
        assert!(a.all);
    }
}
