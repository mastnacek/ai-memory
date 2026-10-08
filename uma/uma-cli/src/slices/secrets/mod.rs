//! Secrets: the credential gate as a user-visible surface.
//!
//! **Read-only by construction.** Scanning never mutates anything; the write
//! and supersede slices call this kernel module to *refuse* facts whose text
//! carries credentials, and agents/operator can scan any text here first.

use anyhow::Result;
use clap::{Args, Subcommand};
use uma_core::secrets::{self, Severity};

#[derive(Args, Debug, Clone)]
pub struct SecretsArgs {
    #[command(subcommand)]
    pub command: SecretsCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SecretsCommand {
    /// Scan text for credentials and injection signatures (read-only)
    Scan(ScanArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    /// The text to scan (joined if several arguments are given)
    pub text: Vec<String>,

    /// Emit findings as JSON
    #[arg(long = "json")]
    pub json: bool,

    /// Read the text from stdin instead of an argument
    #[arg(long = "stdin")]
    pub stdin: bool,
}

/// Executes the Secrets vertical slice.
pub fn run(args: SecretsArgs) -> Result<()> {
    match args.command {
        SecretsCommand::Scan(scan) => scan_text(scan),
    }
}

fn scan_text(args: ScanArgs) -> Result<()> {
    let text = if args.stdin {
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        args.text.join(" ")
    };

    if text.is_empty() {
        anyhow::bail!("Nothing to scan: pass text or --stdin");
    }

    let literals: Vec<String> = secrets::env_secret_literals(&env_literals());
    let findings = secrets::scan(&text, &literals);

    if args.json {
        let payload = serde_json::json!({
            "blocked": secrets::is_blocked(&findings),
            "findings": findings,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if findings.is_empty() {
        println!("No findings — text is clean.");
        return Ok(());
    }

    for finding in &findings {
        let tag = match finding.severity {
            Severity::Block => "BLOCK",
            Severity::Warning => "warn ",
        };
        println!("[{tag}] {}: {}", finding.label, finding.preview);
    }
    if secrets::is_blocked(&findings) {
        println!("\nRefusing this text as memory content — rephrase without the secret.");
    }
    Ok(())
}

/// The process environment's secret values, as (name, value) pairs.
///
/// Spawned agents inherit pi's environment, so a key sitting in `OPENAI_API_KEY`
/// is recognized wherever the operator pasted it.
fn env_literals() -> Vec<(String, String)> {
    std::env::vars().collect()
}

#[cfg(test)]
mod tests;
