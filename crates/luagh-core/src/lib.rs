//! # luagh-core
//!
//! Core types shared across all LuaGH crates: spans, severity levels,
//! diagnostics, symbol kinds, and Lua version definitions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Position & Span
// ---------------------------------------------------------------------------

/// A position in a source file (0-based internally).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// 0-based line number.
    pub line: u32,
    /// 0-based column (byte offset within the line).
    pub column: u32,
    /// Absolute byte offset from start of file.
    pub offset: u32,
}

impl Position {
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Display as 1-based line:column for human output.
    pub fn display_line(&self) -> u32 {
        self.line + 1
    }

    pub fn display_column(&self) -> u32 {
        self.column + 1
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.display_line(), self.display_column())
    }
}

/// A range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn single(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// Diagnostic severity level. Ordered: Hint < Info < Warning < Error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Hint,
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Hint => "hint",
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }

    pub fn sarif_level(&self) -> &'static str {
        match self {
            Severity::Hint => "note",
            Severity::Info => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hint" => Ok(Severity::Hint),
            "info" => Ok(Severity::Info),
            "warning" | "warn" => Ok(Severity::Warning),
            "error" | "err" => Ok(Severity::Error),
            _ => Err(format!("unknown severity: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Lua Version
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LuaVersion {
    #[serde(rename = "lua51")]
    Lua51,
    #[serde(rename = "lua52")]
    Lua52,
    #[serde(rename = "lua53")]
    Lua53,
    #[default]
    #[serde(rename = "lua54")]
    Lua54,
    #[serde(rename = "luajit")]
    LuaJIT,
    #[serde(rename = "luau")]
    Luau,
}

impl LuaVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            LuaVersion::Lua51 => "lua51",
            LuaVersion::Lua52 => "lua52",
            LuaVersion::Lua53 => "lua53",
            LuaVersion::Lua54 => "lua54",
            LuaVersion::LuaJIT => "luajit",
            LuaVersion::Luau => "luau",
        }
    }
}

impl fmt::Display for LuaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for LuaVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lua51" | "lua5.1" | "5.1" => Ok(LuaVersion::Lua51),
            "lua52" | "lua5.2" | "5.2" => Ok(LuaVersion::Lua52),
            "lua53" | "lua5.3" | "5.3" => Ok(LuaVersion::Lua53),
            "lua54" | "lua5.4" | "5.4" => Ok(LuaVersion::Lua54),
            "luajit" | "jit" => Ok(LuaVersion::LuaJIT),
            "luau" => Ok(LuaVersion::Luau),
            _ => Err(format!("unknown Lua version: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Symbol Kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    LocalVariable,
    GlobalVariable,
    Function,
    Method,
    Parameter,
    Label,
    Field,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::LocalVariable => "local_variable",
            SymbolKind::GlobalVariable => "global_variable",
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Parameter => "parameter",
            SymbolKind::Label => "label",
            SymbolKind::Field => "field",
        }
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Diagnostic
// ---------------------------------------------------------------------------

/// A structured diagnostic produced by a rule.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable rule identifier, e.g. "lint.unused_local".
    pub rule_id: String,
    /// Severity level.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// File path (relative or absolute).
    pub file_path: PathBuf,
    /// Source span.
    pub span: Span,
    /// Optional short suggestion (displayed inline).
    pub suggestion: Option<String>,
    /// Optional longer help text.
    pub help: Option<String>,
    /// Source excerpt for display.
    pub source_excerpt: Option<String>,
    /// Structured autofix.
    pub fix: Option<Fix>,
}

impl Diagnostic {
    pub fn new(
        rule_id: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        file_path: impl Into<PathBuf>,
        span: Span,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            message: message.into(),
            file_path: file_path.into(),
            span,
            suggestion: None,
            help: None,
            source_excerpt: None,
            fix: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_source_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.source_excerpt = Some(excerpt.into());
        self
    }
}

/// A structured fix with one or more text edits.
#[derive(Debug, Clone)]
pub struct Fix {
    pub description: String,
    pub edits: Vec<TextEdit>,
}

/// A single text replacement.
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
}

// ---------------------------------------------------------------------------
// Rule Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleCategory {
    Syntax,
    Lint,
    Naming,
    Style,
    Semantic,
}

impl RuleCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleCategory::Syntax => "syntax",
            RuleCategory::Lint => "lint",
            RuleCategory::Naming => "naming",
            RuleCategory::Style => "style",
            RuleCategory::Semantic => "semantic",
        }
    }
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Line Index (for efficient line/column computation)
// ---------------------------------------------------------------------------

/// Precomputed line start offsets for efficient byte-offset to line/column
/// conversion.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offsets of each line start. `line_starts[0]` is always 0.
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Build a line index from source text.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (i, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    /// Convert a byte offset to a `Position`.
    pub fn position(&self, offset: u32) -> Position {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        };
        let column = offset - self.line_starts[line];
        Position {
            line: line as u32,
            column,
            offset,
        }
    }

    /// Get the source text for a given 0-based line number.
    pub fn line_text<'a>(&self, source: &'a str, line: u32) -> Option<&'a str> {
        let start = *self.line_starts.get(line as usize)? as usize;
        let end = self
            .line_starts
            .get(line as usize + 1)
            .map(|&s| s as usize)
            .unwrap_or(source.len());
        // Strip trailing newline
        let text = &source[start..end];
        Some(text.trim_end_matches('\n').trim_end_matches('\r'))
    }

    /// Total number of lines.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

// ---------------------------------------------------------------------------
// Standard Library Globals
// ---------------------------------------------------------------------------

/// Returns the set of standard global names for a given Lua version.
pub fn std_globals(version: LuaVersion) -> Vec<&'static str> {
    let mut globals = vec![
        // Core functions
        "assert",
        "collectgarbage",
        "dofile",
        "error",
        "getmetatable",
        "ipairs",
        "load",
        "loadfile",
        "next",
        "pairs",
        "pcall",
        "print",
        "rawequal",
        "rawget",
        "rawlen",
        "rawset",
        "require",
        "select",
        "setmetatable",
        "tonumber",
        "tostring",
        "type",
        "xpcall",
        // Standard modules (as globals)
        "coroutine",
        "debug",
        "io",
        "math",
        "os",
        "package",
        "string",
        "table",
        // Special globals
        "_G",
        "_VERSION",
    ];

    match version {
        LuaVersion::Lua51 => {
            globals.extend([
                "getfenv",
                "setfenv",
                "loadstring",
                "module",
                "newproxy",
                "unpack",
            ]);
        }
        LuaVersion::Lua52 => {
            globals.extend(["rawlen", "utf8"]);
        }
        LuaVersion::Lua53 => {
            globals.extend(["utf8"]);
        }
        LuaVersion::Lua54 => {
            globals.extend(["utf8", "warn"]);
        }
        LuaVersion::LuaJIT => {
            globals.extend([
                "getfenv",
                "setfenv",
                "loadstring",
                "unpack",
                "newproxy",
                "jit",
                "bit",
                "ffi",
            ]);
        }
        LuaVersion::Luau => {
            globals.extend([
                "typeof",
                "task",
                "buffer",
                "unpack",
                "gcinfo",
                "newproxy",
                "game",
                "workspace",
                "script",
                "shared",
                "Instance",
                "Vector3",
                "Vector2",
                "CFrame",
                "Color3",
                "BrickColor",
                "UDim",
                "UDim2",
                "Enum",
                "Axes",
                "Faces",
                "Ray",
                "Region3",
                "tick",
                "time",
                "wait",
                "delay",
                "spawn",
                "warn",
            ]);
        }
    }

    globals.sort_unstable();
    globals.dedup();
    globals
}

// ---------------------------------------------------------------------------
// Output Format
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "sarif" => Ok(OutputFormat::Sarif),
            _ => Err(format!("unknown output format: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Hint < Severity::Info);
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_severity_parse() {
        assert_eq!("warning".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("error".parse::<Severity>().unwrap(), Severity::Error);
        assert_eq!("hint".parse::<Severity>().unwrap(), Severity::Hint);
        assert!("bogus".parse::<Severity>().is_err());
    }

    #[test]
    fn test_lua_version_parse() {
        assert_eq!("lua54".parse::<LuaVersion>().unwrap(), LuaVersion::Lua54);
        assert_eq!("luajit".parse::<LuaVersion>().unwrap(), LuaVersion::LuaJIT);
        assert_eq!("luau".parse::<LuaVersion>().unwrap(), LuaVersion::Luau);
        assert_eq!("5.1".parse::<LuaVersion>().unwrap(), LuaVersion::Lua51);
    }

    #[test]
    fn test_line_index() {
        let source = "hello\nworld\nfoo";
        let idx = LineIndex::new(source);
        assert_eq!(idx.line_count(), 3);

        let pos = idx.position(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        let pos = idx.position(6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);

        let pos = idx.position(8);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);

        assert_eq!(idx.line_text(source, 0), Some("hello"));
        assert_eq!(idx.line_text(source, 1), Some("world"));
        assert_eq!(idx.line_text(source, 2), Some("foo"));
    }

    #[test]
    fn test_diagnostic_builder() {
        let diag = Diagnostic::new(
            "lint.unused_local",
            Severity::Warning,
            "unused local variable `x`",
            "test.lua",
            Span::default(),
        )
        .with_suggestion("prefix with `_` to suppress")
        .with_help("unused variables may indicate dead code");

        assert_eq!(diag.rule_id, "lint.unused_local");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.suggestion.is_some());
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_std_globals_contains_print() {
        for version in [
            LuaVersion::Lua51,
            LuaVersion::Lua52,
            LuaVersion::Lua53,
            LuaVersion::Lua54,
            LuaVersion::LuaJIT,
            LuaVersion::Luau,
        ] {
            let globals = std_globals(version);
            assert!(globals.contains(&"print"), "print missing for {version}");
        }
    }
}
