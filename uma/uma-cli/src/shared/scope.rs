use anyhow::Result;
use uma_core::{domain::Scope, store::Store};

/// Resolves a CLI scope argument into a `Scope` domain model.
/// If `Some("global")` -> `Scope::Global`.
/// If `Some(name)` -> `Scope::Project(name)`.
/// If `None` -> tries to detect git root for the project name; falls back to `Scope::Global`.
pub fn resolve_scope(scope_arg: Option<String>) -> Result<Scope> {
    match scope_arg {
        Some(s) if s.eq_ignore_ascii_case("global") => Ok(Scope::Global),
        Some(s) => Ok(Scope::Project(s)),
        None => match Store::find_git_root() {
            Ok(git_root) => {
                let project_name = git_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(Scope::Project(project_name))
            }
            Err(_) => Ok(Scope::Global),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_explicit_global() {
        let scope = resolve_scope(Some("global".to_string())).unwrap();
        assert_eq!(scope, Scope::Global);
    }

    #[test]
    fn test_resolve_explicit_project() {
        let scope = resolve_scope(Some("custom-repo".to_string())).unwrap();
        assert_eq!(scope, Scope::Project("custom-repo".to_string()));
    }
}
