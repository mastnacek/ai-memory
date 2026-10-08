mod checks;
mod findings;

use anyhow::{bail, Result};
use clap::Args;
use serde_json::json;
use uma_core::store::Store;

use checks::{count_fact_files, inspect_index, root_finding};
use findings::{fail, ok, warn, Finding, Level};

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Emit the report as JSON
    #[arg(long = "json")]
    pub json: bool,

    /// Exit non-zero when any check fails
    #[arg(long = "strict")]
    pub strict: bool,
}

/// Executes the Doctor vertical slice: a read-only health report on the store and
/// its rebuildable index cache.
///
/// It reports; it never repairs. The index is opened read-only, so a diagnosis
/// cannot drop or rebuild the very thing it is inspecting. Remedies are printed
/// as commands for the operator to run deliberately, which keeps a health check
/// from becoming an unrequested mutation.
pub fn run(args: DoctorArgs) -> Result<()> {
    let mut findings = Vec::new();

    let project_root = Store::find_git_root().ok().map(|root| root.join(".uma"));
    let global_root = Store::global_root_path().ok();

    match &project_root {
        Some(root) => findings.push(root_finding("project store", root)),
        None => findings.push(warn(
            "project store",
            "Not inside a git repository, so there is no project scope here.",
            "Run `uma list --scope global`, or run inside a repository.",
        )),
    }
    match &global_root {
        Some(root) => findings.push(root_finding("global store", root)),
        None => findings.push(fail(
            "global store",
            "Could not resolve the user profile data directory.",
            "Check the APPDATA / XDG_DATA_HOME environment variables.",
        )),
    }

    let disk_facts =
        count_fact_files(project_root.as_deref()) + count_fact_files(global_root.as_deref());
    findings.push(ok(
        "fact files",
        format!("{disk_facts} Markdown document(s) across both scopes"),
    ));

    let db_path = Store::central_db_path()?;
    if db_path.exists() {
        let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
        findings.push(ok(
            "index database",
            format!("{} ({size} bytes)", db_path.display()),
        ));
        inspect_index(&db_path, disk_facts, &mut findings);
    } else {
        findings.push(warn(
            "index database",
            format!("{} does not exist yet", db_path.display()),
            "Build it with `uma search \"\" --reindex`.",
        ));
    }

    report(&findings, &args)?;

    let failed = findings.iter().filter(|f| f.level == Level::Fail).count();
    if args.strict && failed > 0 {
        bail!("{failed} health check(s) failed.");
    }
    Ok(())
}

fn report(findings: &[Finding], args: &DoctorArgs) -> Result<()> {
    let failed = findings.iter().filter(|f| f.level == Level::Fail).count();
    let warned = findings.iter().filter(|f| f.level == Level::Warn).count();

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "failed": failed,
                "warnings": warned,
                "findings": findings.iter().map(|f| json!({
                    "level": f.level.label(),
                    "check": f.label,
                    "detail": f.detail,
                    "remedy": f.remedy,
                })).collect::<Vec<_>>(),
            }))?
        );
        return Ok(());
    }

    println!("UMA doctor — read-only health report\n");
    for finding in findings {
        println!(
            "[{}] {}: {}",
            finding.level.label(),
            finding.label,
            finding.detail
        );
        if let Some(remedy) = finding.remedy {
            println!("        → {remedy}");
        }
    }
    println!(
        "\n{} check(s): {} ok, {warned} warning(s), {failed} failure(s).",
        findings.len(),
        findings.len() - warned - failed
    );
    println!("Nothing was modified.");
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
        Doctor(DoctorArgs),
    }

    #[test]
    fn test_doctor_defaults_to_reporting_only() {
        let cli = TestCli::try_parse_from(["uma", "doctor"]).expect("should parse");
        match cli.command {
            Top::Doctor(args) => {
                assert!(!args.json);
                assert!(!args.strict, "must not fail by default — it only reports");
            }
        }
    }

    #[test]
    fn test_missing_root_counts_as_zero() {
        assert_eq!(count_fact_files(Some(std::path::Path::new("/nope"))), 0);
        assert_eq!(count_fact_files(None), 0);
    }
}
