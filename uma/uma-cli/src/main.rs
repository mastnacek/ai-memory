use anyhow::Result;
use clap::{Parser, Subcommand};
use std::str::FromStr;
use uma_core::{
    domain::{Fact, FactId, FactType, Scope},
    store::Store,
};

#[derive(Parser)]
#[command(
    name = "uma",
    version,
    about = "Universal Memory Assistant - Personal knowledge management"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write a new fact
    Write {
        /// Fact type (decision, pattern, reference, note, task, or custom)
        #[arg(long, default_value = "note")]
        r#type: String,

        /// Fact title
        #[arg(short = 'T', long)]
        title: String,

        /// Fact body (if not provided, reads from stdin)
        #[arg(short, long)]
        body: Option<String>,

        /// Scope (project name or "global")
        #[arg(short, long)]
        scope: Option<String>,

        /// Tags (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// Read a fact by ID
    Read {
        /// Fact ID (ULID)
        id: String,
    },

    /// List facts
    List {
        /// Scope (project name or "global")
        #[arg(short, long)]
        scope: Option<String>,

        /// Filter by fact type
        #[arg(short, long)]
        r#type: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Write {
            r#type,
            title,
            body,
            scope,
            tags,
        } => {
            let fact_type = FactType::from_str(&r#type)?;
            let body = body.unwrap_or_else(|| {
                use std::io::{self, Read};
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer).unwrap();
                buffer.trim().to_string()
            });

            let scope = resolve_scope(scope)?;
            let mut fact = Fact::new(scope, fact_type, title, body);
            fact.tags = tags;

            let store = get_store(&fact.scope)?;
            store.write(&fact)?;
            println!("Created fact: {}", fact.id);
        }
        Commands::Read { id } => {
            let fact_id = FactId::from_str(&id)?;
            let store = get_global_store()?;
            let fact = store.read_by_id(&fact_id)?;
            print_fact(&fact);
        }
        Commands::List { scope, r#type } => {
            let scope = resolve_scope(scope)?;
            let fact_type = r#type.map(|t| FactType::from_str(&t)).transpose()?;
            let store = get_store(&scope)?;
            let facts = store.list(&scope, fact_type.as_ref())?;

            if facts.is_empty() {
                println!("No facts found.");
            } else {
                for fact in facts {
                    println!("{} [{}] {}", fact.id, fact.fact_type, fact.title);
                }
            }
        }
    }

    Ok(())
}

fn resolve_scope(scope_arg: Option<String>) -> Result<Scope> {
    match scope_arg {
        Some(s) if s == "global" => Ok(Scope::Global),
        Some(s) => Ok(Scope::Project(s)),
        None => {
            // Try to detect project from git root
            match Store::find_git_root() {
                Ok(git_root) => {
                    let project_name = git_root
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    Ok(Scope::Project(project_name))
                }
                Err(_) => Ok(Scope::Global),
            }
        }
    }
}

fn get_store(scope: &Scope) -> Result<Store> {
    match scope {
        Scope::Global => Store::global(),
        Scope::Project(name) => Store::project(name.clone()),
    }
}

fn get_global_store() -> Result<Store> {
    Store::global()
}

fn print_fact(fact: &Fact) {
    println!("ID:       {}", fact.id);
    println!("Scope:    {}", fact.scope);
    println!("Type:     {}", fact.fact_type);
    println!("Title:    {}", fact.title);
    println!(
        "Since:    {}",
        fact.validity.since.format("%Y-%m-%d %H:%M:%S UTC")
    );
    if let Some(until) = fact.validity.until {
        println!("Until:    {}", until.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if !fact.tags.is_empty() {
        println!("Tags:     {}", fact.tags.join(", "));
    }
    if !fact.links.is_empty() {
        println!(
            "Links:    {}",
            fact.links
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();
    println!("{}", fact.body);
}
