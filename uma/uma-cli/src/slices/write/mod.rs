use anyhow::Result;
use clap::Args;
use std::io::{self, Read};
use std::str::FromStr;
use uma_core::domain::{Fact, FactType};

use crate::shared::{scope::resolve_scope, store_helper::get_store};

#[derive(Args, Debug, Clone)]
pub struct WriteArgs {
    /// Fact type (decision, preference, fact, skill, correction, note, pattern, reference, task)
    #[arg(short = 't', long = "type", default_value = "note")]
    pub fact_type: String,

    /// Fact title
    #[arg(short = 'T', long = "title")]
    pub title: String,

    /// Optional one-line description (OKF format)
    #[arg(short = 'd', long = "desc")]
    pub description: Option<String>,

    /// Fact body (if not provided, reads from stdin)
    #[arg(short = 'b', long = "body")]
    pub body: Option<String>,

    /// Scope (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Tags (comma-separated)
    #[arg(long = "tags", value_delimiter = ',')]
    pub tags: Vec<String>,

    /// Invocation template for `skill` facts (placeholders like {{tag}}).
    /// Stored as data only — UMA expands it elsewhere and never executes it.
    #[arg(long = "template")]
    pub template: Option<String>,
}

/// Executes the Write vertical slice: creates and stores a new fact.
pub fn run(args: WriteArgs) -> Result<()> {
    let fact_type = FactType::from_str(&args.fact_type)?;
    let body = match args.body {
        Some(b) => b,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer.trim().to_string()
        }
    };

    let scope = resolve_scope(args.scope)?;
    let mut fact = Fact::new(scope, fact_type, args.title, body);
    fact.description = args.description;
    fact.tags = args.tags;
    fact.template = args.template;

    let store = get_store(&fact.scope)?;
    store.write(&fact)?;
    println!("Created fact: {}", fact.id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(flatten)]
        args: WriteArgs,
    }

    #[test]
    fn test_write_args_parsing() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from([
            "test",
            "--type",
            "decision",
            "--title",
            "Use VSA Architecture",
            "--body",
            "All components follow vertical slices.",
            "--scope",
            "global",
            "--tags",
            "vsa,arch,rust",
        ])?;

        assert_eq!(cli.args.fact_type, "decision");
        assert_eq!(cli.args.title, "Use VSA Architecture");
        assert_eq!(
            cli.args.body,
            Some("All components follow vertical slices.".to_string())
        );
        assert_eq!(cli.args.scope, Some("global".to_string()));
        assert_eq!(cli.args.tags, vec!["vsa", "arch", "rust"]);
        Ok(())
    }
}
