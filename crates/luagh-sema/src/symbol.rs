//! Symbol table model.

use luagh_core::{Span, SymbolKind};

/// Symbol identifier (index into SymbolTable).
pub type SymbolId = u32;

/// A symbol (variable, function, parameter, etc.) defined in the source.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    /// Name as written in source.
    pub name: String,
    /// Classified kind.
    pub kind: SymbolKind,
    /// Span of the definition site.
    pub def_span: Span,
    /// Scope in which this symbol is defined.
    pub scope_id: u32,
    /// Spans where this symbol is referenced (reads).
    pub uses: Vec<Span>,
    /// Whether this is a function parameter.
    pub is_parameter: bool,
    /// Whether this is an implicit symbol (e.g. `self` from colon syntax).
    pub is_implicit: bool,
}

impl Symbol {
    /// Returns true if this symbol has been used (read) at least once.
    pub fn is_used(&self) -> bool {
        !self.uses.is_empty()
    }

    /// Returns true if this name starts with `_` (conventional "unused" marker).
    pub fn is_underscore_prefixed(&self) -> bool {
        self.name.starts_with('_') && self.name != "_G" && self.name != "_ENV" && self.name != "_VERSION"
    }
}

/// Collection of all symbols in a file.
#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Add a symbol, returning its ID.
    pub fn add(&mut self, symbol: Symbol) -> SymbolId {
        let id = self.symbols.len() as SymbolId;
        self.symbols.push(symbol);
        id
    }

    /// Get a symbol by ID.
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id as usize)
    }

    /// Get a mutable symbol by ID.
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id as usize)
    }

    /// Iterate all symbols.
    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }

    /// Number of symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// All symbols of a given kind.
    pub fn by_kind(&self, kind: SymbolKind) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter().filter(move |s| s.kind == kind)
    }

    /// All unused local symbols (excluding underscore-prefixed and implicit).
    pub fn unused_locals(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter().filter(|s| {
            matches!(
                s.kind,
                SymbolKind::LocalVariable | SymbolKind::Function | SymbolKind::Parameter
            ) && !s.is_used()
                && !s.is_underscore_prefixed()
                && !s.is_implicit
        })
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luagh_core::{Position, Span};

    fn make_symbol(name: &str, kind: SymbolKind) -> Symbol {
        Symbol {
            id: 0,
            name: name.to_string(),
            kind,
            def_span: Span::new(Position::new(0, 0, 0), Position::new(0, 1, 1)),
            scope_id: 0,
            uses: Vec::new(),
            is_parameter: false,
            is_implicit: false,
        }
    }

    #[test]
    fn test_symbol_table_add_and_get() {
        let mut table = SymbolTable::new();
        let sym = make_symbol("x", SymbolKind::LocalVariable);
        let id = table.add(sym);
        assert_eq!(table.get(id).unwrap().name, "x");
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_unused_locals() {
        let mut table = SymbolTable::new();

        // Unused local
        table.add(make_symbol("unused_var", SymbolKind::LocalVariable));

        // Used local
        let mut used = make_symbol("used_var", SymbolKind::LocalVariable);
        used.uses.push(Span::default());
        table.add(used);

        // Underscore-prefixed (suppressed)
        table.add(make_symbol("_temp", SymbolKind::LocalVariable));

        let unused: Vec<&Symbol> = table.unused_locals().collect();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].name, "unused_var");
    }
}
