use anyhow::Result;
use clap::Args;
use serde_json::json;
use uma_core::store::Store;

#[derive(Args, Debug, Clone)]
pub struct ScopesArgs {
    /// Emit the report as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Scopes vertical slice: discovery for cross-project memory.
///
/// Project memory is deliberately isolated — `list`, `read` and unscoped
/// `search` only see the current project plus global. Isolation is right, but it
/// made cross-project recall unreachable: without this command, an agent could
/// not even *name* another project's scope to query it with `--scope`.
pub fn run(args: ScopesArgs) -> Result<()> {
    let summaries = Store::known_scopes()?;

    if args.json {
        let payload: Vec<_> = summaries
            .iter()
            .map(|summary| {
                json!({
                    "scope": summary.scope.to_string(),
                    "facts": summary.facts,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if summaries.is_empty() {
        println!("No indexed scopes. Write a fact first with `uma write`.");
        return Ok(());
    }

    println!("Known scopes (from the index):\n");
    for summary in &summaries {
        println!(
            "  {:<28} {:>3} fact(s)",
            summary.scope.to_string(),
            summary.facts
        );
    }
    println!("\nCross-project recall: `uma search \"…\" --scope <scope>`;");
    println!("`uma read <id>` resolves any fact whose ID you already hold.");
    Ok(())
}