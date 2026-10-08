//! Recall-slice tests: argument surface and the two transports.

use super::*;
use clap::{Parser, Subcommand};

fn parse(args: &[&str]) -> Result<RecallArgs, clap::Error> {
    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        command: Top,
    }

    #[derive(Subcommand)]
    enum Top {
        Recall(RecallArgs),
    }

    let all = ["uma"]
        .iter()
        .chain(args.iter())
        .copied()
        .collect::<Vec<_>>();
    let parsed = Cli::try_parse_from(all)?;
    match parsed.command {
        Top::Recall(args) => Ok(args),
    }
}

#[test]
fn test_check_parses_message_and_flags() -> Result<(), clap::Error> {
    let args = parse(&[
        "recall", "check", "why", "did", "we", "--judge", "jev", "--json",
    ])?;
    match args.command {
        RecallCommand::Check(check) => {
            assert_eq!(check.prompt, vec!["why", "did", "we"]);
            assert_eq!(check.judge, "jev");
            assert!(check.json);
        }
    }
    Ok(())
}

#[test]
fn test_check_refuses_empty_message() -> Result<(), clap::Error> {
    let args = parse(&["recall", "check"])?;
    match args.command {
        RecallCommand::Check(check) => {
            assert!(
                check_prompt(check).is_err(),
                "empty message must be refused"
            );
        }
    }
    Ok(())
}

#[test]
fn test_offline_verdict_is_deterministic() -> anyhow::Result<()> {
    let args = parse(&["recall", "check", "again like last time with pnpm"])?;
    match args.command {
        RecallCommand::Check(check) => {
            let verdict = fastbrain::judge_recall_need(&check.prompt.join(" "), Judge::Offline)
                .map_err(|f| anyhow::anyhow!("{f}"))?;
            assert!(verdict.answer.search);
            assert_eq!(verdict.judged_by, fastbrain::Backend::Offline);
        }
    }
    Ok(())
}
