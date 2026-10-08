use anyhow::Result;
use clap::Args;
use std::io::{self, Read};
use std::str::FromStr;
use uma_core::{
    domain::{Fact, FactId, FactType},
    store::Store,
};

use crate::shared::{scope::resolve_scope, store_helper::get_store};

#[derive(Args, Debug, Clone)]
pub struct SupersedeArgs {
    /// ID of the predecessor fact to supersede (ULID)
    pub old_id: String,

    /// New fact title
    #[arg(short = 'T', long = "title")]
    pub title: String,

    /// Optional one-line description (OKF format)
    #[arg(short = 'd', long = "desc")]
    pub description: Option<String>,

    /// New fact body (if not provided, reads from stdin)
    #[arg(short = 'b', long = "body")]
    pub body: Option<String>,

    /// Fact type (defaults to predecessor's type if not specified)
    #[arg(short = 't', long = "type")]
    pub fact_type: Option<String>,

    /// Scope (project name or "global", defaults to predecessor's scope)
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Tags (comma-separated, defaults to predecessor's tags if empty)
    #[arg(long = "tags", value_delimiter = ',')]
    pub tags: Vec<String>,
}

/// Executes the Supersede vertical slice: chains supersession and invalidates previous fact.
pub fn run(args: SupersedeArgs) -> Result<()> {
    let old_id = FactId::from_str(&args.old_id)?;
    let old_fact = Store::find_by_id(&old_id)?;

    let fact_type = match args.fact_type {
        Some(t) => FactType::from_str(&t)?,
        None => old_fact.fact_type.clone(),
    };

    let scope = match args.scope {
        Some(s) => resolve_scope(Some(s))?,
        None => old_fact.scope.clone(),
    };

    let tags = if args.tags.is_empty() {
        old_fact.tags.clone()
    } else {
        args.tags
    };

    let body = match args.body {
        Some(b) => b,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer.trim().to_string()
        }
    };

    let mut new_fact = Fact::new(scope.clone(), fact_type, args.title, body);
    new_fact.description = args.description;
    new_fact.tags = tags;

    let store = get_store(&scope)?;
    let created = store.supersede(&old_id, new_fact)?;

    println!("Superseded fact {} with new fact: {}", old_id, created.id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(flatten)]
        args: SupersedeArgs,
    }

    #[test]
    fn test_supersede_args_parsing() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from([
            "test",
            "01M4D7S5YART7AGWN7RDSRNRM1",
            "--title",
            "New Revised Architecture Decision",
            "--desc",
            "Updated architecture for S4",
        ])?;

        assert_eq!(cli.args.old_id, "01M4D7S5YART7AGWN7RDSRNRM1");
        assert_eq!(cli.args.title, "New Revised Architecture Decision");
        assert_eq!(
            cli.args.description,
            Some("Updated architecture for S4".to_string())
        );
        Ok(())
    }
}
