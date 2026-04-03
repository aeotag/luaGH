//! Rule trait — the core abstraction for all LuaGH checks.

use luagh_core::{Diagnostic, RuleCategory, Severity};

use crate::context::RuleContext;

/// A single analysis rule. All built-in and custom rules implement this trait.
///
/// Rules are expected to be stateless — all state comes through [`RuleContext`].
/// They must be `Send + Sync` to support parallel file analysis.
pub trait Rule: Send + Sync {
    /// Stable identifier in `category.name` format, e.g. `"lint.unused_local"`.
    fn id(&self) -> &'static str;

    /// Human-readable name, e.g. `"Unused Local Variable"`.
    fn name(&self) -> &'static str;

    /// One-line description.
    fn description(&self) -> &'static str;

    /// Extended help text with examples, shown by `luagh explain <rule-id>`.
    fn help(&self) -> &'static str {
        ""
    }

    /// Default severity (can be overridden in config).
    fn default_severity(&self) -> Severity;

    /// Rule category for grouping in `luagh rules` output.
    fn category(&self) -> RuleCategory;

    /// Run the rule against a parsed+analyzed file, returning diagnostics.
    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic>;
}
