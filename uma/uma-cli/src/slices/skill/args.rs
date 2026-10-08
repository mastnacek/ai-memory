use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub command: SkillCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SkillCommand {
    /// Create a skill fact that holds an invocation template
    New(NewArgs),
    /// List available skills and the placeholders they need
    List(ListArgs),
    /// Show one skill and its template
    Show(LookupArgs),
    /// Expand a skill template (never executes it)
    Invoke(InvokeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct NewArgs {
    /// Skill name; becomes the fact title and the lookup key
    #[arg(long = "name")]
    pub name: String,

    /// Invocation template. Placeholders are written {{like_this}}
    #[arg(long = "template")]
    pub template: String,

    /// One-line description (OKF `description`)
    #[arg(long = "desc")]
    pub description: Option<String>,

    /// Comma-separated tags
    #[arg(long = "tags")]
    pub tags: Option<String>,

    /// Markdown body explaining when to reach for this skill
    #[arg(short = 'b', long = "body")]
    pub body: Option<String>,

    /// Scope (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    /// Scope (project name or "global"). Defaults to the current project.
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Emit as JSON
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct LookupArgs {
    /// Skill name or fact ID
    pub name_or_id: String,

    /// Scope to search first (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Emit as JSON
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct InvokeArgs {
    /// Skill name or fact ID
    pub name_or_id: String,

    /// Placeholder value, repeatable: --set tag=v1
    #[arg(long = "set", value_name = "KEY=VALUE")]
    pub set: Vec<String>,

    /// Scope to search first (project name or "global")
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Emit as JSON
    #[arg(long = "json")]
    pub json: bool,
}
