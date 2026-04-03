//! Rule: `lint.shadowing`
//!
//! Reports local variables that shadow a variable from an outer scope.

use luagh_core::{Diagnostic, RuleCategory, Severity, SymbolKind};

use crate::context::RuleContext;
use crate::rule::Rule;

/// Detects variable shadowing.
pub struct Shadowing;

impl Rule for Shadowing {
    fn id(&self) -> &'static str {
        "lint.shadowing"
    }

    fn name(&self) -> &'static str {
        "Variable Shadowing"
    }

    fn description(&self) -> &'static str {
        "Reports local variables that shadow a variable from an outer scope"
    }

    fn help(&self) -> &'static str {
        r#"Shadowing occurs when a local variable in an inner scope has the same
name as a variable in an outer scope. This can lead to confusion about
which variable is being referenced.

Example:
  local x = 1
  do
      local x = 2   -- warning: `x` shadows a variable in an outer scope
      print(x)
  end
  print(x)           -- this still refers to the outer x

To suppress: rename one of the variables, or disable this rule.
"#
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Lint
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // For each scope, check if any defined symbol shadows a symbol
        // in a parent scope.
        for scope in ctx.scopes.iter() {
            if let Some(parent_id) = scope.parent {
                for (name, &_sym_id) in &scope.symbols {
                    // Skip underscore-prefixed and common patterns
                    if name.starts_with('_') || name == "self" {
                        continue;
                    }

                    // Check if the name exists in any ancestor scope
                    if ctx.scopes.lookup(name, parent_id).is_some() {
                        // Get the symbol for span info
                        if let Some(sym) = ctx.symbols.get(_sym_id) {
                            // Only warn about local variables and parameters
                            if matches!(
                                sym.kind,
                                SymbolKind::LocalVariable
                                    | SymbolKind::Parameter
                                    | SymbolKind::Function
                            ) {
                                let mut diag = Diagnostic::new(
                                    self.id(),
                                    self.default_severity(),
                                    format!("`{}` shadows a variable from an outer scope", name),
                                    ctx.file_path,
                                    sym.def_span,
                                )
                                .with_help("consider renaming to avoid confusion".to_string());

                                if let Some(line) = ctx.source_line(sym.def_span.start.line) {
                                    diag = diag.with_source_excerpt(line.to_string());
                                }

                                diagnostics.push(diag);
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_metadata() {
        let rule = Shadowing;
        assert_eq!(rule.id(), "lint.shadowing");
        assert_eq!(rule.category(), RuleCategory::Lint);
        assert_eq!(rule.default_severity(), Severity::Warning);
    }
}
