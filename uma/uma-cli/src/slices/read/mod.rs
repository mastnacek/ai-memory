use anyhow::Result;
use clap::Args;
use std::str::FromStr;
use uma_core::{domain::FactId, store::Store};

use crate::shared::format::print_fact;

#[derive(Args, Debug, Clone)]
pub struct ReadArgs {
    /// Fact ID (ULID)
    pub id: String,

    /// Emit the fact as JSON instead of a human-readable summary
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Read vertical slice: finds and displays a fact by its ID.
pub fn run(args: ReadArgs) -> Result<()> {
    let fact_id = FactId::from_str(&args.id)?;
    let fact = Store::find_by_id(&fact_id)?;

    if args.json {
        println!("{}", serde_json::to_string(&fact)?);
    } else {
        print_fact(&fact);
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
        args: ReadArgs,
    }

    #[test]
    fn test_read_args_parsing() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from(["test", "01M4D6K8J6QGDFC0Y11FW45R87"])?;
        assert_eq!(cli.args.id, "01M4D6K8J6QGDFC0Y11FW45R87");
        assert!(!cli.args.json);

        let cli_json = TestCli::try_parse_from(["test", "01M4D6K8J6QGDFC0Y11FW45R87", "--json"])?;
        assert!(cli_json.args.json);
        Ok(())
    }
}
