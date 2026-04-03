//! Rule context — the shared data passed to each rule during analysis.

use std::path::Path;

use full_moon::ast::Ast;

use luagh_config::{Config, NamingConfig};
use luagh_core::{LineIndex, LuaVersion};
use luagh_sema::{ScopeTree, SymbolTable};

/// Context provided to each rule's `check()` method.
///
/// Contains the parsed AST, semantic model, configuration, and utilities
/// needed for analysis. Rules should treat this as read-only.
pub struct RuleContext<'a> {
    /// Path of the file being analyzed.
    pub file_path: &'a Path,
    /// Original source text.
    pub source: &'a str,
    /// Parsed AST.
    pub ast: &'a Ast,
    /// Symbol table from semantic analysis.
    pub symbols: &'a SymbolTable,
    /// Scope tree from semantic analysis.
    pub scopes: &'a ScopeTree,
    /// Project configuration.
    pub config: &'a Config,
    /// Active Lua version.
    pub lua_version: LuaVersion,
    /// Line index for offset/position conversions.
    pub line_index: &'a LineIndex,
}

impl<'a> RuleContext<'a> {
    /// Check whether a rule is enabled (not set to "off" in config).
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        match self.config.rules.get(rule_id) {
            Some(override_val) => !override_val.is_off(),
            None => true,
        }
    }

    /// Get the naming config, applying any path-specific overrides.
    pub fn naming_config(&self) -> &NamingConfig {
        // TODO: Check per-path overrides and merge
        &self.config.naming
    }

    /// Get the source line for a given 0-based line number.
    pub fn source_line(&self, line: u32) -> Option<&str> {
        self.line_index.line_text(self.source, line)
    }

    /// Check if a global name is known (standard library or configured).
    pub fn is_known_global(&self, name: &str) -> bool {
        let std_globals: std::collections::HashSet<&str> =
            luagh_core::std_globals(self.lua_version)
                .into_iter()
                .collect();
        std_globals.contains(name) || self.config.globals.is_known(name)
    }
}
