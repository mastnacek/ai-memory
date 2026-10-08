use anyhow::Result;
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
}

/// Executes the List vertical slice: retrieves and displays a summary list of facts.
pub fn run(args: ListArgs) -> Result<()> {
    let scope = resolve_scope(args.scope)?;
    let fact_type = args.fact_type.map(|t| FactType::from_str(&t)).transpose()?;
    let store = get_store(&scope)?;
    let facts = store.list(&scope, fact_type.as_ref())?;

    if facts.is_empty() {
        println!("No facts found.");
    } else {
        for fact in facts {
            print_fact_summary(&fact);
        }
    }
    Ok(())
}
