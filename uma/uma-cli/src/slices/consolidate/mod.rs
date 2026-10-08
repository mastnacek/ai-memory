use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::str::FromStr;
use uma_core::consolidate::{analyze, AnalyzeOptions, ConsolidationReport, ContradictionPair};
use uma_core::domain::{Fact, FactType};
use uma_core::fastbrain;

use crate::shared::{scope::resolve_scope, store_helper::get_store};

#[derive(Args, Debug, Clone)]
pub struct ConsolidateArgs {
    /// Scope (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Only consider facts of this type (decision, preference, pattern, ...)
    #[arg(short = 't', long = "type")]
    pub fact_type: Option<String>,

    /// Similarity threshold (0.0-1.0) above which a merge is proposed
    #[arg(long = "threshold", default_value_t = 0.55)]
    pub threshold: f64,

    /// Weight of title similarity against body similarity (0.0-1.0)
    #[arg(long = "title-weight", default_value_t = 0.6)]
    pub title_weight: f64,

    /// Include deprecated facts in the analysis
    #[arg(long = "include-deprecated")]
    pub include_deprecated: bool,

    /// Relationship judge: off = deterministic heuristics, jev = Jev via
    /// OpenRouter (degrades to heuristics with a note when unavailable)
    #[arg(long = "judge", default_value = "off", value_parser = ["off", "jev"])]
    pub judge: String,

    /// Accepted for parity with the documented workflow; consolidation never applies changes
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Emit the report as JSON
    #[arg(long = "json")]
    pub json: bool,
}

/// Executes the Consolidate vertical slice: proposes merges and flags conflicts.
///
/// This slice only ever reads. Applying a proposal is the caller's decision and
/// happens through `supersede` (retire the loser) or `write` (state the merged
/// result), which is what keeps consolidation behind the same consent rules as
/// any other memory mutation.
pub fn run(args: ConsolidateArgs) -> Result<()> {
    let scope = resolve_scope(args.scope)?;
    let fact_type = args
        .fact_type
        .as_deref()
        .map(FactType::from_str)
        .transpose()?;
    let store = get_store(&scope)?;
    let facts = store.list(&scope, fact_type.as_ref())?;

    let opts = AnalyzeOptions {
        threshold: args.threshold.clamp(0.0, 1.0),
        title_weight: args.title_weight.clamp(0.0, 1.0),
        include_deprecated: args.include_deprecated,
    };
    let mut report = analyze(&facts, &opts);
    if args.judge == "jev" {
        refine_with_judge(&mut report, &facts);
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!(
        "Consolidation report — scanned {} active fact(s) in {}\n",
        report.scanned, scope
    );

    if report.is_empty() {
        println!("No consolidation candidates found.");
        println!("Nothing was modified.");
        return Ok(());
    }

    if !report.duplicate_groups.is_empty() {
        println!("Proposed merges ({}):", report.duplicate_groups.len());
        for (idx, group) in report.duplicate_groups.iter().enumerate() {
            println!("{}. score {:.2} — {}", idx + 1, group.score, group.reason);
            for (id, title) in group.ids.iter().zip(group.titles.iter()) {
                println!("   - {} {}", id, title);
            }
            println!(
                "   → Keep one and supersede it onto the rest: `uma supersede <kept-id> --title \"...\" --body \"...\"\n"
            );
        }
    }

    if !report.contradictions.is_empty() {
        println!("Contradictions ({}):", report.contradictions.len());
        for (idx, pair) in report.contradictions.iter().enumerate() {
            println!("{}. score {:.2} — {}", idx + 1, pair.score, pair.reason);
            println!("   - {} {}", pair.left, pair.left_title);
            println!("   - {} {}", pair.right, pair.right_title);
            println!("   → Supersede whichever you no longer hold.\n");
        }
    }

    println!("Nothing was modified — review, then apply with `uma supersede` or `uma write`.");
    Ok(())
}

/// Refines duplicate groups semantically: a pair the lexical pass merged but
/// the judge calls a contradiction moves to the contradictions list.
///
/// Read-only like everything in this slice — the report is proposals, and
/// applying one is still a supersession through the modal.
fn refine_with_judge(report: &mut ConsolidationReport, facts: &[Fact]) {
    let by_id: HashMap<uma_core::domain::FactId, &Fact> = facts.iter().map(|f| (f.id, f)).collect();
    for group in &mut report.duplicate_groups {
        let mut kept = Vec::with_capacity(group.ids.len());
        let mut members = group.ids.iter();
        if let Some(first) = members.next() {
            kept.push(*first);
            for id in members {
                let Some(right) = by_id.get(id) else {
                    kept.push(*id);
                    continue;
                };
                let Some(left_fact) = by_id.get(first) else {
                    kept.push(*id);
                    continue;
                };
                let text = format!(
                    "{}
{}",
                    left_fact.title, left_fact.body
                );
                let other = format!(
                    "{}
{}",
                    right.title, right.body
                );
                match fastbrain::judge_relationship(&text, &other, fastbrain::Judge::Jev) {
                    Ok(judgment) => {
                        if judgment.answer == fastbrain::Relationship::Contradiction {
                            report.contradictions.push(ContradictionPair {
                                left: *first,
                                right: *id,
                                left_title: left_fact.title.clone(),
                                right_title: right.title.clone(),
                                score: judgment.confidence,
                                reason: format!(
                                    "Jev judge: opposite rules ({}; {})",
                                    judgment.judged_by.as_str(),
                                    judgment.notes.as_deref().unwrap_or("")
                                ),
                            });
                            continue;
                        }
                    }
                    Err(failure) => {
                        eprintln!("note: judge unavailable ({failure}); keeping lexical verdict");
                    }
                }
                kept.push(*id);
            }
        }
        group.ids = kept;
    }
    report.duplicate_groups.retain(|g| g.ids.len() >= 2);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(flatten)]
        args: ConsolidateArgs,
    }

    #[test]
    fn test_defaults_match_documented_workflow() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from(["test"])?;
        assert!((cli.args.threshold - 0.55).abs() < f64::EPSILON);
        assert!((cli.args.title_weight - 0.6).abs() < f64::EPSILON);
        assert!(!cli.args.json);
        assert!(!cli.args.include_deprecated);
        Ok(())
    }

    #[test]
    fn test_dry_run_and_json_are_accepted() -> Result<(), clap::Error> {
        let cli = TestCli::try_parse_from(["test", "--scope", "global", "--dry-run", "--json"])?;
        assert!(cli.args.dry_run);
        assert!(cli.args.json);
        assert_eq!(cli.args.scope.as_deref(), Some("global"));
        Ok(())
    }
}
