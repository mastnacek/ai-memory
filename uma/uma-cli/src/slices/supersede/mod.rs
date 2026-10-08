use anyhow::Result;
use clap::Args;
use std::io::{self, Read};
use std::str::FromStr;
use uma_core::{
    domain::{Fact, FactId, FactType},
    store::Store,
};

use crate::shared::{parse::parse_datetime_or_date, scope::resolve_scope, store_helper::get_store};

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

    /// Invocation template (defaults to the predecessor's template).
    /// Without this, a revision of a `skill` fact would silently lose its
    /// template and the skill would stop being expandable.
    #[arg(long = "template")]
    pub template: Option<String>,

    /// When the claim stops being trusted without re-verification.
    /// Defaults to the predecessor's `stale_after` (RFC 3339 or a bare date).
    /// Pass an extended date to re-verify the fact, or note that inheriting a
    /// date that has already passed would make the revision stale at birth.
    #[arg(long = "stale-after", value_name = "WHEN")]
    pub stale_after: Option<String>,
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
    // Inherit like type/scope/tags: a revision must never silently drop the
    // invocation template, or the skill would stop being invokable.
    new_fact.template = args.template.or_else(|| old_fact.template.clone());
    // Same for the validity deadline — but a re-verification SHOULD pass a new
    // one, or the revision inherits a date that may already have passed.
    new_fact.validity.stale_after = match args.stale_after {
        Some(raw) => Some(parse_datetime_or_date(&raw)?),
        None => old_fact.validity.stale_after,
    };

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
