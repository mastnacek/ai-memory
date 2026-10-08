//! Secrets-slice tests: argument surface and the refusal contract.

use super::*;
use clap::{Parser, Subcommand};

fn parse(args: &[&str]) -> Result<SecretsArgs, clap::Error> {
    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        command: Top,
    }

    #[derive(Subcommand)]
    enum Top {
        Secrets(SecretsArgs),
    }

    let all = ["uma"]
        .iter()
        .chain(args.iter())
        .copied()
        .collect::<Vec<_>>();
    let parsed = Cli::try_parse_from(all)?;
    match parsed.command {
        Top::Secrets(args) => Ok(args),
    }
}

#[test]
fn test_scan_parses_text_and_flags() -> Result<(), clap::Error> {
    let args = parse(&["secrets", "scan", "some", "text", "--json"])?;
    match args.command {
        SecretsCommand::Scan(scan) => {
            assert_eq!(scan.text, vec!["some", "text"]);
            assert!(scan.json);
            assert!(!scan.stdin);
        }
    }
    Ok(())
}

#[test]
fn test_scan_refuses_empty_input() -> Result<(), clap::Error> {
    let args = parse(&["secrets", "scan"])?;
    match args.command {
        SecretsCommand::Scan(scan) => {
            let result = scan_text(scan);
            assert!(result.is_err(), "empty scan input must be refused");
        }
    }
    Ok(())
}
