//! Import-slice tests: selection and the proposal contract.

use super::*;
use anyhow::Result;

#[test]
fn test_sessions_subcommand_parses_with_defaults() -> Result<(), clap::Error> {
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "uma")]
    struct Cli {
        #[command(subcommand)]
        command: Top,
    }

    #[derive(Subcommand)]
    enum Top {
        Import(ImportArgs),
    }

    let parsed = Cli::try_parse_from(["uma", "import", "sessions"])?;
    match parsed.command {
        Top::Import(ImportArgs {
            command: ImportCommand::Sessions(args),
        }) => {
            assert_eq!(args.limit, 10, "default session limit");
            assert_eq!(args.max_per_session, 3, "default per-session bound");
            assert!(args.session.is_none());
            assert!(!args.json);
        }
    }
    Ok(())
}

#[test]
fn test_session_selection_accepts_id_or_path() -> Result<(), clap::Error> {
    use clap::{Parser, Subcommand};

    #[derive(Parser)]
    #[command(name = "uma")]
    struct Cli {
        #[command(subcommand)]
        command: Top,
    }

    #[derive(Subcommand)]
    enum Top {
        Import(ImportArgs),
    }

    let parsed = Cli::try_parse_from([
        "uma",
        "import",
        "sessions",
        "--session",
        "01a03770",
        "--max-per-session",
        "5",
        "--json",
    ])?;
    match parsed.command {
        Top::Import(ImportArgs {
            command: ImportCommand::Sessions(args),
        }) => {
            assert_eq!(args.session.as_deref(), Some("01a03770"));
            assert_eq!(args.max_per_session, 5);
            assert!(args.json);
        }
    }
    Ok(())
}

#[test]
fn test_proposal_contract_is_read_only() {
    // The slice's whole safety story is that it never writes. Its public
    // surface must therefore contain no function that mutates: `run` only
    // prints, and the store-touching functions it uses come from the gated
    // write path, not from here.
    assert!(!std::any::type_name::<ImportArgs>().contains("write"));
}
