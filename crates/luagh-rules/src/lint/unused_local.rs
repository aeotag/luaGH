//! Rule: `lint.unused_local`
//!
//! Reports local variables and parameters that are declared but never read.

use luagh_core::{Diagnostic, RuleCategory, Severity, SymbolKind};

use crate::context::RuleContext;
use crate::rule::Rule;

/// Detects unused local variables.
pub struct UnusedLocal;

impl Rule for UnusedLocal {
    fn id(&self) -> &'static str {
        "lint.unused_local"
    }

    fn name(&self) -> &'static str {
        "Unused Local Variable"
    }

    fn description(&self) -> &'static str {
        "Reports local variables that are declared but never used"
    }

    fn help(&self) -> &'static str {
        r#"This rule detects local variables that are assigned a value but never
read anywhere in their scope.

To suppress this warning:
  - Prefix the variable with `_` (e.g., `local _unused = compute()`)
  - Disable the rule in luagh.toml: `"lint.unused_local" = "off"`

Example:
  local x = 1      -- warning: unused local variable `x`
  local _y = 2     -- ok: underscore prefix suppresses the warning
  local z = 3
  print(z)          -- ok: z is used
"#
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Lint
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        ctx.symbols
            .unused_locals()
            .filter(|sym| {
                // Skip parameters unless explicitly configured
                !sym.is_parameter
                    && matches!(
                        sym.kind,
                        SymbolKind::LocalVariable | SymbolKind::Function
                    )
            })
            .map(|sym| {
                let mut diag = Diagnostic::new(
                    self.id(),
                    self.default_severity(),
                    format!("unused local variable `{}`", sym.name),
                    ctx.file_path,
                    sym.def_span,
                )
                .with_suggestion("prefix with `_` to suppress this warning".to_string());

                // Add source excerpt if available
                if let Some(line) = ctx.source_line(sym.def_span.start.line) {
                    diag = diag.with_source_excerpt(line.to_string());
                }

                diag
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_metadata() {
        let rule = UnusedLocal;
        assert_eq!(rule.id(), "lint.unused_local");
        assert_eq!(rule.category(), RuleCategory::Lint);
        assert_eq!(rule.default_severity(), Severity::Warning);
    }
}
