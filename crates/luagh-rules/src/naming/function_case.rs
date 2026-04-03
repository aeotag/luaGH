//! Rules: `naming.function_case` and `naming.method_case`
//!
//! Checks that function and method names follow the configured naming convention.

use luagh_core::{Diagnostic, RuleCategory, Severity, SymbolKind};

use crate::context::RuleContext;
use crate::naming::convention::{describe_pattern, NamingConventionEngine};
use crate::rule::Rule;

// ---------------------------------------------------------------------------
// naming.function_case
// ---------------------------------------------------------------------------

pub struct FunctionCase;

impl Rule for FunctionCase {
    fn id(&self) -> &'static str {
        "naming.function_case"
    }

    fn name(&self) -> &'static str {
        "Function Naming Convention"
    }

    fn description(&self) -> &'static str {
        "Checks that function names follow the configured pattern (default: PascalCase)"
    }

    fn help(&self) -> &'static str {
        r#"By default, functions should use PascalCase:
  local function ProcessData() end    -- ok
  local function process_data() end   -- warning

This applies to:
  - `local function Name() end`
  - `function Name() end`
  - `local Name = function() end` (when detectable)

Configure the pattern in luagh.toml:
  [naming]
  function = "^[A-Z][A-Za-z0-9]*$"
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
            Err(_) => return Vec::new(),
        };

        ctx.symbols
            .by_kind(SymbolKind::Function)
            .filter_map(|sym| {
                let violation = engine.check(&sym.name, SymbolKind::Function)?;
                let pattern_desc = engine
                    .pattern_for(SymbolKind::Function)
                    .map(describe_pattern)
                    .unwrap_or("configured pattern");

                let mut diag = Diagnostic::new(
                    self.id(),
                    self.default_severity(),
                    format!(
                        "function `{}` should be {pattern_desc}",
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
// naming.method_case
// ---------------------------------------------------------------------------

pub struct MethodCase;

impl Rule for MethodCase {
    fn id(&self) -> &'static str {
        "naming.method_case"
    }

    fn name(&self) -> &'static str {
        "Method Naming Convention"
    }

    fn description(&self) -> &'static str {
        "Checks that method names (colon syntax) follow the configured pattern (default: PascalCase)"
    }

    fn help(&self) -> &'static str {
        r#"By default, methods (using colon syntax) should use PascalCase:
  function tbl:GetName() end     -- ok
  function tbl:get_name() end    -- warning

Configure the pattern in luagh.toml:
  [naming]
  method = "^[A-Z][A-Za-z0-9]*$"
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
            Err(_) => return Vec::new(),
        };

        ctx.symbols
            .by_kind(SymbolKind::Method)
            .filter_map(|sym| {
                // For methods like `tbl:MethodName`, we check only the method
                // name part (after the colon).
                let method_name = sym
                    .name
                    .rsplit_once(':')
                    .map(|(_, name)| name)
                    .unwrap_or(&sym.name);

                let violation = engine.check(method_name, SymbolKind::Method)?;
                let pattern_desc = engine
                    .pattern_for(SymbolKind::Method)
                    .map(describe_pattern)
                    .unwrap_or("configured pattern");

                let mut diag = Diagnostic::new(
                    self.id(),
                    self.default_severity(),
                    format!(
                        "method `{method_name}` should be {pattern_desc}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_case_metadata() {
        let rule = FunctionCase;
        assert_eq!(rule.id(), "naming.function_case");
        assert_eq!(rule.category(), RuleCategory::Naming);
    }

    #[test]
    fn test_method_case_metadata() {
        let rule = MethodCase;
        assert_eq!(rule.id(), "naming.method_case");
        assert_eq!(rule.category(), RuleCategory::Naming);
    }
}
