use anyhow::Result;
use chrono::Utc;
use clap::Args;
use std::str::FromStr;
use uma_core::domain::FactType;

use crate::shared::{format::print_fact_summary, scope::resolve_scope, store_helper::get_store};

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    /// Scope (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Filter by fact type (decision, preference, fact, skill, note, etc.)
    #[arg(short = 't', long = "type")]
    pub fact_type: Option<String>,

    /// Include deprecated/superseded facts (hidden by default)
    #[arg(long = "include-deprecated")]
    pub include_deprecated: bool,
}

/// Executes the List vertical slice: retrieves and displays a summary list of facts.
pub fn run(args: ListArgs) -> Result<()> {
    let scope = resolve_scope(args.scope)?;
    let fact_type = args.fact_type.map(|t| FactType::from_str(&t)).transpose()?;
    let store = get_store(&scope)?;
    let mut facts = store.list(&scope, fact_type.as_ref())?;

    // Deprecated/superseded facts are hidden unless explicitly requested: showing a
    // replaced rule next to its replacement is exactly the misleading-memory hazard
    // supersession exists to prevent.
    let stored = facts.len();
    if !args.include_deprecated {
        let now = Utc::now();
        facts.retain(|fact| fact.is_active_at(now));
    }

    if facts.is_empty() {
        if stored > 0 {
            println!(
                "No active facts found ({} hidden as deprecated/inactive; pass --include-deprecated to show them).",
                stored
            );
        } else {
            println!("No facts found.");
        }
    } else {
        for fact in facts {
            print_fact_summary(&fact);
        }
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
        args: ListArgs,
    }

    #[test]
    fn test_list_args_parsing() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from(["test", "--scope", "global", "--type", "decision"])?;
        assert_eq!(cli.args.scope.as_deref(), Some("global"));
        assert_eq!(cli.args.fact_type.as_deref(), Some("decision"));
        assert!(!cli.args.include_deprecated);
        Ok(())
    }

    #[test]
    fn test_list_include_deprecated() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from(["test", "--include-deprecated"])?;
        assert!(cli.args.include_deprecated);
        assert!(cli.args.scope.is_none());
        Ok(())
    }
}
