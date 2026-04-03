//! Rule: `lint.undefined_global`
//!
//! Reports reads/writes to global names not in the standard library or config.

use luagh_core::{Diagnostic, RuleCategory, Severity, SymbolKind};

use crate::context::RuleContext;
use crate::rule::Rule;

/// Detects use of undefined global variables.
pub struct UndefinedGlobal;

impl Rule for UndefinedGlobal {
    fn id(&self) -> &'static str {
        "lint.undefined_global"
    }

    fn name(&self) -> &'static str {
        "Undefined Global"
    }

    fn description(&self) -> &'static str {
        "Reports global variables that are not defined in the standard library or configuration"
    }

    fn help(&self) -> &'static str {
        r#"This rule triggers when a global variable is used but is not part of the
Lua standard library for the configured version, nor listed in the [globals]
section of luagh.toml.

To fix:
  - If the global is intentional, add it to [globals] in luagh.toml:
      [globals]
      rw = ["my_global"]
  - If it's a typo, fix the name
  - If it should be local, add `local` before the declaration

Example:
  prnt("hello")         -- error: use of undefined global `prnt`
  print("hello")        -- ok: `print` is in the standard library
  MY_FRAMEWORK.init()   -- add MY_FRAMEWORK to [globals] if intentional
"#
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Lint
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        ctx.symbols
            .by_kind(SymbolKind::GlobalVariable)
            .filter(|sym| !ctx.is_known_global(&sym.name))
            .map(|sym| {
                let mut diag = Diagnostic::new(
                    self.id(),
                    self.default_severity(),
                    format!("use of undefined global `{}`", sym.name),
                    ctx.file_path,
                    sym.def_span,
                )
                .with_help(
                    "add this global to [globals] in luagh.toml, or define it as a local"
                        .to_string(),
                );

                // Try to suggest similar standard globals
                if let Some(suggestion) = find_similar_global(&sym.name, ctx) {
                    diag = diag.with_suggestion(format!("did you mean `{suggestion}`?"));
                }

                if let Some(line) = ctx.source_line(sym.def_span.start.line) {
                    diag = diag.with_source_excerpt(line.to_string());
                }

                diag
            })
            .collect()
    }
}

/// Simple edit-distance-based suggestion for typos.
fn find_similar_global(name: &str, ctx: &RuleContext) -> Option<String> {
    let std_globals = luagh_core::std_globals(ctx.lua_version);
    let name_lower = name.to_lowercase();

    std_globals
        .into_iter()
        .filter(|g| {
            let g_lower = g.to_lowercase();
            // Simple heuristic: names within edit distance 2 or sharing a long common prefix
            let dist = levenshtein(&name_lower, &g_lower);
            dist > 0 && dist <= 2
        })
        .min_by_key(|g| levenshtein(&name_lower, &g.to_lowercase()))
        .map(|s| s.to_string())
}

/// Simple Levenshtein distance implementation for short strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost)
                .min(prev[j + 1] + 1)
                .min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("print", "prnt"), 1);
        assert_eq!(levenshtein("print", "print"), 0);
        assert_eq!(levenshtein("table", "tabel"), 2);
        assert_eq!(levenshtein("", "abc"), 3);
    }

    #[test]
    fn test_rule_metadata() {
        let rule = UndefinedGlobal;
        assert_eq!(rule.id(), "lint.undefined_global");
        assert_eq!(rule.default_severity(), Severity::Error);
    }
}
