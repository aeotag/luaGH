//! # luagh-sema
//!
//! Semantic analysis for LuaGH. Provides scope tracking, symbol table
//! construction, and name resolution over a parsed Lua AST.

pub mod resolver;
pub mod scope;
pub mod symbol;

pub use resolver::Resolver;
pub use scope::{Scope, ScopeId, ScopeKind, ScopeTree};
pub use symbol::{Symbol, SymbolId, SymbolTable};

use luagh_core::{Diagnostic, LuaVersion};
use luagh_parser::ParsedFile;

/// Result of semantic analysis on a single file.
#[derive(Debug)]
pub struct SemanticModel {
    pub scopes: ScopeTree,
    pub symbols: SymbolTable,
    pub diagnostics: Vec<Diagnostic>,
}

/// Run semantic analysis on a parsed file.
pub fn analyze(parsed: &ParsedFile, lua_version: LuaVersion) -> SemanticModel {
    let mut resolver = Resolver::new(lua_version, &parsed.line_index);
    resolver.analyze(&parsed.ast, &parsed.path);

    let (scopes, symbols, diagnostics) = resolver.into_parts();

    SemanticModel {
        scopes,
        symbols,
        diagnostics,
    }
}
