//! # luagh-rules
//!
//! Rule engine, built-in lint rules, and naming convention checks for LuaGH.

pub mod context;
pub mod lint;
pub mod naming;
pub mod registry;
pub mod rule;

pub use context::RuleContext;
pub use registry::RuleRegistry;
pub use rule::Rule;
