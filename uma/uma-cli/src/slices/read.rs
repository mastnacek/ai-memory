use anyhow::Result;
use clap::Args;
use std::str::FromStr;
use uma_core::{domain::FactId, store::Store};

use crate::shared::format::print_fact;

#[derive(Args, Debug, Clone)]
pub struct ReadArgs {
    /// Fact ID (ULID)
    pub id: String,
}

/// Executes the Read vertical slice: finds and displays a fact by its ID.
pub fn run(args: ReadArgs) -> Result<()> {
    let fact_id = FactId::from_str(&args.id)?;
    let fact = Store::find_by_id(&fact_id)?;
    print_fact(&fact);
    Ok(())
}
