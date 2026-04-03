//! Rules: `naming.local_variable_case` and `naming.global_variable_case`
//!
//! Checks that variable names follow the configured naming convention.

use luagh_core::{Diagnostic, RuleCategory, Severity, SymbolKind};

use crate::context::RuleContext;
use crate::naming::convention::{describe_pattern, NamingConventionEngine};
use crate::rule::Rule;

// ---------------------------------------------------------------------------
// naming.local_variable_case
// ---------------------------------------------------------------------------

pub struct LocalVariableCase;

impl Rule for LocalVariableCase {
    fn id(&self) -> &'static str {
        "naming.local_variable_case"
    }

    fn name(&self) -> &'static str {
        "Local Variable Naming Convention"
    }

    fn description(&self) -> &'static str {
        "Checks that local variable names follow the configured pattern (default: snake_case)"
    }

    fn help(&self) -> &'static str {
        r#"By default, local variables should use snake_case:
  local my_variable = 1    -- ok
  local MyVariable = 1     -- warning

Configure the pattern in luagh.toml:
  [naming]
  local_variable = "^[a-z_][a-z0-9_]*$"

Names in the `ignore_names` list are always exempt.
"#
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Naming
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let naming_config = ctx.naming_config();
        let engine = match NamingConventionEngine::from_config(naming_config) {
            Ok(e) => e,
            Err(_) => return Vec::new(), // Invalid regex in config, skip
        };

        ctx.symbols
            .by_kind(SymbolKind::LocalVariable)
            .filter_map(|sym| {
                let violation = engine.check(&sym.name, SymbolKind::LocalVariable)?;
                let pattern_desc = engine
                    .pattern_for(SymbolKind::LocalVariable)
                    .map(describe_pattern)
                    .unwrap_or("configured pattern");

                let mut diag = Diagnostic::new(
                    self.id(),
                    self.default_severity(),
                    format!(
                        "local variable `{}` should be {pattern_desc}",
                        sym.name
                    ),
                    ctx.file_path,
                    sym.def_span,
                )
                .with_suggestion(format!(
                    "expected pattern: {}",
                    violation.expected_pattern
                ));

                if let Some(line) = ctx.source_line(sym.def_span.start.line) {
                    diag = diag.with_source_excerpt(line.to_string());
                }

                Some(diag)
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// naming.global_variable_case
// ---------------------------------------------------------------------------

pub struct GlobalVariableCase;

impl Rule for GlobalVariableCase {
    fn id(&self) -> &'static str {
        "naming.global_variable_case"
    }

    fn name(&self) -> &'static str {
        "Global Variable Naming Convention"
    }

    fn description(&self) -> &'static str {
        "Checks that global variable names follow the configured pattern (default: snake_case)"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Naming
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let naming_config = ctx.naming_config();
        let engine = match NamingConventionEngine::from_config(naming_config) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        ctx.symbols
            .by_kind(SymbolKind::GlobalVariable)
            .filter_map(|sym| {
                // Skip known standard globals
                if ctx.is_known_global(&sym.name) {
                    return None;
                }

                let violation = engine.check(&sym.name, SymbolKind::GlobalVariable)?;
                let pattern_desc = engine
                    .pattern_for(SymbolKind::GlobalVariable)
                    .map(describe_pattern)
                    .unwrap_or("configured pattern");

                let mut diag = Diagnostic::new(
                    self.id(),
                    self.default_severity(),
                    format!(
                        "global variable `{}` should be {pattern_desc}",
                        sym.name
                    ),
                    ctx.file_path,
                    sym.def_span,
                )
                .with_suggestion(format!(
                    "expected pattern: {}",
                    violation.expected_pattern
                ));

                if let Some(line) = ctx.source_line(sym.def_span.start.line) {
                    diag = diag.with_source_excerpt(line.to_string());
                }

                Some(diag)
            })
            .collect()
    }
}
