mod args;

pub use args::*;

use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use serde_json::json;
use uma_core::domain::{Fact, FactId, FactType, Scope};
use uma_core::skill::{expand, parse_assignments, placeholders};
use uma_core::store::Store;

use crate::shared::{format::print_fact, scope::resolve_scope, store_helper::get_store};

/// Executes the Skill vertical slice: procedural, template-backed memory.
///
/// The slice stores and expands templates; it never runs them. Expansion output
/// is text only, so executing it stays with whoever holds the approval — a
/// harness shell tool or a human at a terminal.
pub fn run(args: SkillArgs) -> Result<()> {
    match args.command {
        SkillCommand::New(a) => new(a),
        SkillCommand::List(a) => list(a),
        SkillCommand::Show(a) => show(a),
        SkillCommand::Invoke(a) => invoke(a),
    }
}

fn new(args: NewArgs) -> Result<()> {
    if placeholders(&args.template).is_empty() {
        eprintln!("note: template declares no {{placeholders}} — fine for a fixed command.");
    }

    let scope = resolve_scope(args.scope)?;
    let store = get_store(&scope)?;

    let mut fact = Fact::new(
        scope,
        FactType::Skill,
        args.name.clone(),
        args.body
            .unwrap_or_else(|| format!("### Invocation\n\n```\n{}\n```\n", args.template.trim())),
    );
    fact.description = args.description;
    fact.template = Some(args.template);
    if let Some(tags) = args.tags {
        fact.tags = tags
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
    }

    store.write(&fact)?;

    println!("Created skill '{}' ({})", fact.title, fact.id);
    let needed = placeholders(fact.template.as_deref().unwrap_or(""));
    if !needed.is_empty() {
        println!("Placeholders: {}", needed.join(", "));
        println!(
            "Expand with: uma skill invoke {} --set {}=<value>",
            fact.title, needed[0]
        );
    }
    Ok(())
}

fn list(args: ListArgs) -> Result<()> {
    let scope = resolve_scope(args.scope)?;
    let store = get_store(&scope)?;
    let facts = store.list(&scope, Some(&FactType::Skill))?;
    let now = chrono::Utc::now();
    let active: Vec<&Fact> = facts.iter().filter(|f| f.is_active_at(now)).collect();

    if active.is_empty() {
        println!("No skills found in {}.", scope);
        return Ok(());
    }

    if args.json {
        let items: Vec<_> = active
            .iter()
            .map(|f| {
                json!({
                    "id": f.id.to_string(),
                    "name": f.title,
                    "description": f.description,
                    "placeholders": placeholders(f.template.as_deref().unwrap_or("")),
                    "template": f.template,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    for fact in active {
        let needed = placeholders(fact.template.as_deref().unwrap_or(""));
        let suffix = if needed.is_empty() {
            String::new()
        } else {
            format!("  [{}]", needed.join(", "))
        };
        println!("{} {}{}", fact.id, fact.title, suffix);
    }
    Ok(())
}

fn show(args: LookupArgs) -> Result<()> {
    let fact = find_skill(args.scope.as_deref(), &args.name_or_id)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&fact)?);
    } else {
        print_fact(&fact);
    }
    Ok(())
}

fn invoke(args: InvokeArgs) -> Result<()> {
    let fact = find_skill(args.scope.as_deref(), &args.name_or_id)?;
    let template = fact
        .template
        .clone()
        .ok_or_else(|| anyhow!("Skill '{}' has no invocation template.", fact.title))?;

    let vars = parse_assignments(&args.set)?;
    let result = expand(&template, &vars);

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "id": fact.id.to_string(),
                "skill": fact.title,
                "command": result.output,
                "missing": result.missing,
                "unused": result.unused,
                "executed": false,
            }))?
        );
        return Ok(());
    }

    println!("Skill: {} ({})", fact.title, fact.id);
    println!();
    println!("{}", result.output);
    println!();

    if !result.missing.is_empty() {
        println!(
            "Missing placeholders: {} — pass --set name=value",
            result.missing.join(", ")
        );
    }
    if !result.unused.is_empty() {
        println!("Unused --set values (typo?): {}", result.unused.join(", "));
    }
    println!("Not executed by UMA — run the command above yourself, so your own approval applies.");
    Ok(())
}

/// Resolves a skill by fact ID first, then by case-insensitive title.
fn find_skill(scope_name: Option<&str>, key: &str) -> Result<Fact> {
    if let Ok(id) = FactId::from_str(key) {
        if let Ok(fact) = Store::find_by_id(&id) {
            return Ok(fact);
        }
    }

    let scopes: Vec<Scope> = match scope_name {
        Some(name) => vec![resolve_scope(Some(name.to_string()))?],
        None => {
            let mut candidates = Vec::new();
            if let Ok(current) = resolve_scope(None) {
                candidates.push(current);
            }
            candidates.push(Scope::Global);
            candidates
        }
    };

    let now = chrono::Utc::now();
    for scope in scopes {
        let store = get_store(&scope)?;
        for fact in store.list(&scope, Some(&FactType::Skill))? {
            if fact.title.eq_ignore_ascii_case(key) && fact.is_active_at(now) {
                return Ok(fact);
            }
        }
    }

    bail!("No skill found matching '{key}'. Run `uma skill list` to see what is available.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, Subcommand};

    #[derive(Parser, Debug)]
    #[command(name = "uma")]
    struct TestCli {
        #[command(subcommand)]
        command: TopCommand,
    }

    #[derive(Subcommand, Debug)]
    enum TopCommand {
        Skill(SkillArgs),
    }

    fn skill_args(argv: &[&str]) -> SkillArgs {
        let cli = TestCli::try_parse_from(argv).expect("should parse");
        match cli.command {
            TopCommand::Skill(args) => args,
        }
    }

    #[test]
    fn test_skill_new_parsing() {
        match skill_args(&[
            "uma",
            "skill",
            "new",
            "--name",
            "docker-build",
            "--template",
            "docker build -t {{tag}} .",
        ])
        .command
        {
            SkillCommand::New(new) => {
                assert_eq!(new.name, "docker-build");
                assert_eq!(new.template, "docker build -t {{tag}} .");
                assert!(new.scope.is_none());
            }
            _ => panic!("expected New"),
        }
    }

    #[test]
    fn test_skill_invoke_accepts_repeated_sets() {
        match skill_args(&[
            "uma", "skill", "invoke", "docker-build", "--set", "tag=v1", "--set", "env=prod",
        ])
        .command
        {
            SkillCommand::Invoke(invoke) => {
                assert_eq!(invoke.name_or_id, "docker-build");
                assert_eq!(invoke.set, vec!["tag=v1", "env=prod"]);
                assert!(!invoke.json);
            }
            _ => panic!("expected Invoke"),
        }
    }
}
