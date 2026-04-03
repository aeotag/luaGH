//! Naming convention engine — shared logic for all naming rules.

use std::collections::HashMap;
use std::collections::HashSet;

use regex::Regex;

use luagh_core::SymbolKind;
use luagh_config::NamingConfig;

/// A naming convention violation.
#[derive(Debug)]
pub struct NamingViolation {
    pub name: String,
    pub kind: SymbolKind,
    pub expected_pattern: String,
}

/// Engine that checks names against configured patterns.
pub struct NamingConventionEngine {
    patterns: HashMap<SymbolKind, Regex>,
    ignore_names: HashSet<String>,
}

impl NamingConventionEngine {
    /// Build the engine from a NamingConfig.
    pub fn from_config(config: &NamingConfig) -> Result<Self, regex::Error> {
        let mut patterns = HashMap::new();

        if let Some(ref p) = config.local_variable {
            patterns.insert(SymbolKind::LocalVariable, Regex::new(p)?);
        }
        if let Some(ref p) = config.global_variable {
            patterns.insert(SymbolKind::GlobalVariable, Regex::new(p)?);
        }
        if let Some(ref p) = config.function {
            patterns.insert(SymbolKind::Function, Regex::new(p)?);
        }
        if let Some(ref p) = config.method {
            patterns.insert(SymbolKind::Method, Regex::new(p)?);
        }
        if let Some(ref p) = config.constant {
            // Note: constants share the same regex but are checked separately
            // against symbols classified as constants by the semantic analysis.
            patterns.insert(SymbolKind::Field, Regex::new(p)?); // placeholder
        }
        if let Some(ref p) = config.parameter {
            patterns.insert(SymbolKind::Parameter, Regex::new(p)?);
        }

        let ignore_names: HashSet<String> = config.ignore_names.iter().cloned().collect();

        Ok(Self {
            patterns,
            ignore_names,
        })
    }

    /// Check a name against the pattern for its symbol kind.
    /// Returns `Some(violation)` if the name doesn't match, `None` if it's ok.
    pub fn check(&self, name: &str, kind: SymbolKind) -> Option<NamingViolation> {
        // Skip ignored names
        if self.ignore_names.contains(name) {
            return None;
        }

        // Skip metamethods (double-underscore prefix)
        if name.starts_with("__") {
            return None;
        }

        // Skip single underscore (discard variable)
        if name == "_" {
            return None;
        }

        if let Some(pattern) = self.patterns.get(&kind) {
            if !pattern.is_match(name) {
                return Some(NamingViolation {
                    name: name.to_string(),
                    kind,
                    expected_pattern: pattern.to_string(),
                });
            }
        }

        None
    }

    /// Get the pattern for a symbol kind, if configured.
    pub fn pattern_for(&self, kind: SymbolKind) -> Option<&str> {
        self.patterns.get(&kind).map(|r| r.as_str())
    }
}

/// Describe a naming pattern in human terms.
pub fn describe_pattern(pattern: &str) -> &'static str {
    match pattern {
        r"^[a-z_][a-z0-9_]*$" => "snake_case",
        r"^[A-Z][A-Za-z0-9]*$" => "PascalCase",
        r"^[A-Z][A-Z0-9_]*$" => "SCREAMING_SNAKE_CASE",
        r"^[a-z][a-zA-Z0-9]*$" => "camelCase",
        _ => "custom pattern",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luagh_config::NamingConfig;

    fn default_engine() -> NamingConventionEngine {
        NamingConventionEngine::from_config(&NamingConfig::default()).unwrap()
    }

    #[test]
    fn test_snake_case_local() {
        let engine = default_engine();
        assert!(engine
            .check("my_var", SymbolKind::LocalVariable)
            .is_none());
        assert!(engine
            .check("_temp", SymbolKind::LocalVariable)
            .is_none());
        assert!(engine
            .check("MyVar", SymbolKind::LocalVariable)
            .is_some());
        assert!(engine
            .check("myVar", SymbolKind::LocalVariable)
            .is_some());
    }

    #[test]
    fn test_pascal_case_function() {
        let engine = default_engine();
        assert!(engine.check("ProcessData", SymbolKind::Function).is_none());
        assert!(engine.check("Init", SymbolKind::Function).is_none());
        assert!(engine
            .check("process_data", SymbolKind::Function)
            .is_some());
    }

    #[test]
    fn test_ignore_names() {
        let engine = default_engine();
        assert!(engine
            .check("_G", SymbolKind::GlobalVariable)
            .is_none());
        assert!(engine
            .check("self", SymbolKind::Parameter)
            .is_none());
        assert!(engine
            .check("_VERSION", SymbolKind::GlobalVariable)
            .is_none());
    }

    #[test]
    fn test_metamethods_exempt() {
        let engine = default_engine();
        assert!(engine
            .check("__index", SymbolKind::Function)
            .is_none());
        assert!(engine
            .check("__tostring", SymbolKind::Function)
            .is_none());
    }

    #[test]
    fn test_pattern_description() {
        assert_eq!(describe_pattern(r"^[a-z_][a-z0-9_]*$"), "snake_case");
        assert_eq!(describe_pattern(r"^[A-Z][A-Za-z0-9]*$"), "PascalCase");
        assert_eq!(
            describe_pattern(r"^[A-Z][A-Z0-9_]*$"),
            "SCREAMING_SNAKE_CASE"
        );
    }
}
