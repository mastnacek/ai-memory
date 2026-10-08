use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Args;
use serde_json::json;
use uma_core::domain::{Fact, FactType};
use uma_core::serialization::fact_to_markdown;

use crate::shared::{scope::resolve_scope, store_helper::get_store};

#[derive(Args, Debug, Clone)]
pub struct ExportArgs {
    /// Scope (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Only export facts of this type
    #[arg(short = 't', long = "type")]
    pub fact_type: Option<String>,

    /// Include deprecated/superseded facts (default: only active ones)
    #[arg(long = "include-deprecated")]
    pub include_deprecated: bool,

    /// Write an OKF bundle (one Markdown document per fact) instead of printing
    #[arg(long = "okf")]
    pub okf: bool,

    /// Destination directory for an OKF bundle (required with --okf)
    #[arg(long = "out")]
    pub out: Option<PathBuf>,

    /// Emit JSON on stdout (the default when --okf is not given)
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Export vertical slice: produces a portable copy of memory.
///
/// Read-only with respect to the store. It exists so memory is never locked in:
/// an OKF bundle is plain Markdown, readable by any tool and diffable in git.
pub fn run(args: ExportArgs) -> Result<()> {
    if args.okf && args.out.is_none() {
        bail!("--okf writes a bundle and therefore requires --out <directory>.");
    }
    if args.out.is_some() && !args.okf {
        bail!("--out is only meaningful with --okf (otherwise export prints to stdout).");
    }

    let scope = resolve_scope(args.scope)?;
    let fact_type = args
        .fact_type
        .as_deref()
        .map(FactType::from_str)
        .transpose()?;
    let store = get_store(&scope)?;
    let all = store.list(&scope, fact_type.as_ref())?;

    let now = Utc::now();
    let stored = all.len();
    let facts: Vec<Fact> = if args.include_deprecated {
        all
    } else {
        all.into_iter().filter(|f| f.is_active_at(now)).collect()
    };

    if facts.is_empty() {
        println!(
            "Nothing to export from {scope}{}.",
            if stored > 0 {
                " (all matching facts are deprecated; pass --include-deprecated)"
            } else {
                ""
            }
        );
        return Ok(());
    }

    match args.out {
        Some(dest) => write_bundle(&dest, &scope.to_string(), &facts),
        None => {
            let payload: Vec<_> = facts
                .iter()
                .map(|fact| {
                    json!({
                        "id": fact.id.to_string(),
                        "scope": fact.scope.to_string(),
                        "type": fact.fact_type.to_string(),
                        "title": fact.title,
                        "description": fact.description,
                        "tags": fact.tags,
                        "status": fact.status.to_string(),
                        "supersedes": fact.supersedes.map(|s| s.to_string()),
                        "template": fact.template,
                        "since": fact.validity.since.to_rfc3339(),
                        "until": fact.validity.until.map(|u| u.to_rfc3339()),
                        "body": fact.body,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&payload)?);
            Ok(())
        }
    }
}

fn write_bundle(dest: &PathBuf, scope: &str, facts: &[Fact]) -> Result<()> {
    fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create export directory {:?}", dest))?;

    for fact in facts {
        let type_dir = dest.join(fact.fact_type.dir_name());
        fs::create_dir_all(&type_dir)?;
        let path = type_dir.join(format!("{}.md", fact.id));
        let content = fact_to_markdown(fact).context("Failed to serialize fact")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write exported fact to {:?}", path))?;
    }

    let manifest = json!({
        "format": "okf",
        "okf_version": "0.2",
        "exported_by": "uma",
        "exported_at": Utc::now().to_rfc3339(),
        "scope": scope,
        "fact_count": facts.len(),
        "facts": facts.iter().map(|f| json!({
            "id": f.id.to_string(),
            "type": f.fact_type.to_string(),
            "title": f.title,
        })).collect::<Vec<_>>(),
    });
    fs::write(
        dest.join("MANIFEST.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;

    println!(
        "Exported {} fact(s) from {} to {}\nEach fact is a standalone OKF v0.2 Markdown document; MANIFEST.json lists them.",
        facts.len(),
        scope,
        dest.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    #[command(name = "uma")]
    struct TestCli {
        #[command(subcommand)]
        command: Top,
    }

    #[derive(clap::Subcommand, Debug)]
    enum Top {
        Export(ExportArgs),
    }

    fn parse(argv: &[&str]) -> Result<ExportArgs, clap::Error> {
        match TestCli::try_parse_from(argv)?.command {
            Top::Export(a) => Ok(a),
        }
    }

    #[test]
    fn test_export_defaults_to_stdout_json() -> Result<(), clap::Error> {
        let a = parse(&["uma", "export"])?;
        assert!(!a.okf);
        assert!(a.out.is_none());
        assert!(!a.include_deprecated, "deprecated facts are opt-in");
        Ok(())
    }

    #[test]
    fn test_okf_requires_out() {
        let err = run(ExportArgs {
            scope: None,
            fact_type: None,
            include_deprecated: false,
            okf: true,
            out: None,
            json: false,
        })
        .unwrap_err();
        assert!(err.to_string().contains("requires --out"));
    }

    #[test]
    fn test_out_without_okf_is_rejected() {
        let err = run(ExportArgs {
            scope: None,
            fact_type: None,
            include_deprecated: false,
            okf: false,
            out: Some(PathBuf::from("/tmp/nope")),
            json: true,
        })
        .unwrap_err();
        assert!(err.to_string().contains("only meaningful with --okf"));
    }
}
