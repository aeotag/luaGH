//! Scope model for lexical scoping in Lua.

use std::collections::HashMap;

use crate::symbol::SymbolId;

/// Scope identifier (index into ScopeTree).
pub type ScopeId = u32;

/// The kind of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// File-level (module) scope.
    Module,
    /// Function body.
    Function,
    /// Generic block (do..end, if/else body).
    Block,
    /// Loop body (for, while, repeat).
    Loop,
}

/// A single lexical scope.
#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    /// Symbol names defined directly in this scope, mapped to their SymbolId.
    pub symbols: HashMap<String, SymbolId>,
    /// Child scopes.
    pub children: Vec<ScopeId>,
}

impl Scope {
    pub fn new(id: ScopeId, parent: Option<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            id,
            parent,
            kind,
            symbols: HashMap::new(),
            children: Vec::new(),
        }
    }
}

/// Tree of all scopes in a file.
#[derive(Debug)]
pub struct ScopeTree {
    scopes: Vec<Scope>,
}

impl ScopeTree {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Create a new scope, returning its ID.
    pub fn push(&mut self, parent: Option<ScopeId>, kind: ScopeKind) -> ScopeId {
        let id = self.scopes.len() as ScopeId;
        self.scopes.push(Scope::new(id, parent, kind));
        if let Some(pid) = parent {
            self.scopes[pid as usize].children.push(id);
        }
        id
    }

    /// Get a scope by ID.
    pub fn get(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id as usize)
    }

    /// Get a mutable scope by ID.
    pub fn get_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id as usize)
    }

    /// Iterate all scopes.
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }

    /// Look up a name starting from the given scope, walking up parents.
    pub fn lookup(&self, name: &str, from_scope: ScopeId) -> Option<SymbolId> {
        let mut current = Some(from_scope);
        while let Some(sid) = current {
            if let Some(scope) = self.get(sid) {
                if let Some(&sym_id) = scope.symbols.get(name) {
                    return Some(sym_id);
                }
                current = scope.parent;
            } else {
                break;
            }
        }
        None
    }

    /// Number of scopes.
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_tree_basic() {
        let mut tree = ScopeTree::new();
        let root = tree.push(None, ScopeKind::Module);
        let child = tree.push(Some(root), ScopeKind::Function);

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.get(root).unwrap().kind, ScopeKind::Module);
        assert_eq!(tree.get(child).unwrap().parent, Some(root));
        assert!(tree.get(root).unwrap().children.contains(&child));
    }

    #[test]
    fn test_scope_lookup() {
        let mut tree = ScopeTree::new();
        let root = tree.push(None, ScopeKind::Module);
        let child = tree.push(Some(root), ScopeKind::Function);

        // Define "x" in root scope
        tree.get_mut(root).unwrap().symbols.insert("x".to_string(), 0);

        // Lookup "x" from child should find it in parent
        assert_eq!(tree.lookup("x", child), Some(0));

        // Lookup "y" should fail
        assert_eq!(tree.lookup("y", child), None);
    }

    #[test]
    fn test_scope_shadowing() {
        let mut tree = ScopeTree::new();
        let root = tree.push(None, ScopeKind::Module);
        let child = tree.push(Some(root), ScopeKind::Function);

        // Define "x" in both scopes
        tree.get_mut(root).unwrap().symbols.insert("x".to_string(), 0);
        tree.get_mut(child).unwrap().symbols.insert("x".to_string(), 1);

        // Lookup from child should find the inner one
        assert_eq!(tree.lookup("x", child), Some(1));
        // Lookup from root should find the outer one
        assert_eq!(tree.lookup("x", root), Some(0));
    }
}
