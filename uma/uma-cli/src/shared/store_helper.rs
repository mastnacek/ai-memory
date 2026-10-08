use anyhow::Result;
use uma_core::{domain::Scope, store::Store};

/// Gets a `Store` instance for the specified `Scope`.
pub fn get_store(scope: &Scope) -> Result<Store> {
    match scope {
        Scope::Global => Store::global(),
        Scope::Project(name) => Store::project(name.clone()),
    }
}
