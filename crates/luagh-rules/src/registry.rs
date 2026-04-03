//! Rule registry — collects and manages all available rules.

use std::collections::HashMap;

use luagh_core::Diagnostic;

use crate::context::RuleContext;
use crate::lint;
use crate::naming;
use crate::rule::Rule;

/// Registry of all available rules.
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
    by_id: HashMap<String, usize>,
}

impl RuleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            by_id: HashMap::new(),
        }
    }

    /// Register a rule.
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        let idx = self.rules.len();
        self.by_id.insert(rule.id().to_string(), idx);
        self.rules.push(rule);
    }

    /// Create a registry with all built-in rules.
    pub fn builtin() -> Self {
        let mut reg = Self::new();

        // Lint rules
        reg.register(Box::new(lint::unused_local::UnusedLocal));
        reg.register(Box::new(lint::undefined_global::UndefinedGlobal));
        reg.register(Box::new(lint::shadowing::Shadowing));

        // Naming rules
        reg.register(Box::new(naming::variable_case::LocalVariableCase));
        reg.register(Box::new(naming::variable_case::GlobalVariableCase));
        reg.register(Box::new(naming::function_case::FunctionCase));
        reg.register(Box::new(naming::function_case::MethodCase));

        reg
    }

    /// Look up a rule by its ID.
    pub fn get(&self, id: &str) -> Option<&dyn Rule> {
        self.by_id.get(id).map(|&idx| self.rules[idx].as_ref())
    }

    /// Iterate all registered rules.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.iter().map(|r| r.as_ref())
    }

    /// Run all enabled rules against a file context.
    pub fn check_all(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            if ctx.is_rule_enabled(rule.id()) {
                diagnostics.extend(rule.check(ctx));
            }
        }
        diagnostics
    }

    /// Run rules matching a category prefix (e.g., "lint", "naming").
    pub fn check_category(&self, ctx: &RuleContext, category: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            if rule.id().starts_with(category) && ctx.is_rule_enabled(rule.id()) {
                diagnostics.extend(rule.check(ctx));
            }
        }
        diagnostics
    }

    /// Number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry() {
        let reg = RuleRegistry::builtin();
        assert!(reg.len() >= 5, "should have at least 5 built-in rules");

        // Check that key rules exist
        assert!(reg.get("lint.unused_local").is_some());
        assert!(reg.get("lint.undefined_global").is_some());
        assert!(reg.get("lint.shadowing").is_some());
        assert!(reg.get("naming.local_variable_case").is_some());
        assert!(reg.get("naming.function_case").is_some());
    }

    #[test]
    fn test_rule_ids_are_unique() {
        let reg = RuleRegistry::builtin();
        let mut ids: Vec<&str> = reg.iter().map(|r| r.id()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "rule IDs must be unique");
    }
}
