//! # luagh-diagnostics
//!
//! Formatting diagnostics for human-readable text and JSON output.

use std::io::Write;

use luagh_core::{Diagnostic, Severity};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Text Formatter
// ---------------------------------------------------------------------------

/// Format diagnostics as human-readable text output.
pub fn format_text(diagnostics: &[Diagnostic], writer: &mut dyn Write) -> std::io::Result<()> {
    for diag in diagnostics {
        // Header line: severity[rule_id]: message
        writeln!(
            writer,
            "{}[{}]: {}",
            diag.severity, diag.rule_id, diag.message
        )?;

        // Location line
        let line = diag.span.start.display_line();
        let col = diag.span.start.display_column();
        writeln!(writer, "  --> {}:{line}:{col}", diag.file_path.display())?;

        // Source excerpt with underline
        if let Some(ref excerpt) = diag.source_excerpt {
            let line_num = format!("{line}");
            let padding = " ".repeat(line_num.len());
            writeln!(writer, "{padding} |")?;
            writeln!(writer, "{line_num} | {excerpt}")?;

            // Caret underline
            let col_offset = diag.span.start.column as usize;
            let underline_len = if diag.span.end.line == diag.span.start.line {
                (diag.span.end.column as usize)
                    .saturating_sub(col_offset)
                    .max(1)
            } else {
                excerpt.len().saturating_sub(col_offset).max(1)
            };
            let spaces = " ".repeat(col_offset);
            let carets = "^".repeat(underline_len);

            if let Some(ref suggestion) = diag.suggestion {
                writeln!(writer, "{padding} | {spaces}{carets} {suggestion}")?;
            } else {
                writeln!(writer, "{padding} | {spaces}{carets}")?;
            }
        }

        // Help text
        if let Some(ref help) = diag.help {
            let padding = " ".repeat(diag.span.start.display_line().to_string().len());
            writeln!(writer, "{padding} |")?;
            writeln!(writer, "{padding} = help: {help}")?;
        }

        writeln!(writer)?;
    }

    Ok(())
}

/// Format a summary line.
pub fn format_summary(
    diagnostics: &[Diagnostic],
    files_checked: usize,
    writer: &mut dyn Write,
) -> std::io::Result<()> {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let infos = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();
    let hints = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Hint)
        .count();

    let total = diagnostics.len();
    if total == 0 {
        writeln!(writer, "No issues found in {files_checked} file(s).")?;
    } else {
        write!(writer, "Found {total} diagnostic(s)")?;
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(format!("{errors} error(s)"));
        }
        if warnings > 0 {
            parts.push(format!("{warnings} warning(s)"));
        }
        if infos > 0 {
            parts.push(format!("{infos} info"));
        }
        if hints > 0 {
            parts.push(format!("{hints} hint(s)"));
        }
        if !parts.is_empty() {
            write!(writer, " ({})", parts.join(", "))?;
        }
        writeln!(writer, " in {files_checked} file(s)")?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// JSON Formatter
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct JsonOutput {
    pub diagnostics: Vec<JsonDiagnostic>,
    pub summary: JsonSummary,
}

#[derive(Serialize)]
pub struct JsonDiagnostic {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub start: JsonPosition,
    pub end: JsonPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

#[derive(Serialize)]
pub struct JsonPosition {
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize)]
pub struct JsonSummary {
    pub files_checked: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub hints: usize,
}

/// Format diagnostics as JSON.
pub fn format_json(
    diagnostics: &[Diagnostic],
    files_checked: usize,
    writer: &mut dyn Write,
) -> std::io::Result<()> {
    let json_diags: Vec<JsonDiagnostic> = diagnostics
        .iter()
        .map(|d| JsonDiagnostic {
            rule_id: d.rule_id.clone(),
            severity: d.severity.as_str().to_string(),
            message: d.message.clone(),
            file: d.file_path.display().to_string(),
            start: JsonPosition {
                line: d.span.start.display_line(),
                column: d.span.start.display_column(),
            },
            end: JsonPosition {
                line: d.span.end.display_line(),
                column: d.span.end.display_column(),
            },
            suggestion: d.suggestion.clone(),
            help: d.help.clone(),
        })
        .collect();

    let output = JsonOutput {
        summary: JsonSummary {
            files_checked,
            errors: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count(),
            warnings: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count(),
            info: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Info)
                .count(),
            hints: diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Hint)
                .count(),
        },
        diagnostics: json_diags,
    };

    let json = serde_json::to_string_pretty(&output).map_err(std::io::Error::other)?;
    writeln!(writer, "{json}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use luagh_core::{Position, Span};
    use std::path::PathBuf;

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic::new(
            "lint.unused_local",
            Severity::Warning,
            "unused local variable `x`",
            PathBuf::from("test.lua"),
            Span::new(Position::new(4, 6, 40), Position::new(4, 7, 41)),
        )
        .with_suggestion("prefix with `_` to suppress")
        .with_help("unused variables may indicate dead code")
        .with_source_excerpt("local x = 1")
    }

    #[test]
    fn test_format_text() {
        let diags = vec![sample_diagnostic()];
        let mut buf = Vec::new();
        format_text(&diags, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("lint.unused_local"));
        assert!(output.contains("unused local variable `x`"));
        assert!(output.contains("test.lua:5:7"));
        assert!(output.contains("local x = 1"));
        assert!(output.contains("^"));
    }

    #[test]
    fn test_format_json() {
        let diags = vec![sample_diagnostic()];
        let mut buf = Vec::new();
        format_json(&diags, 1, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["diagnostics"][0]["rule_id"], "lint.unused_local");
        assert_eq!(parsed["summary"]["warnings"], 1);
        assert_eq!(parsed["summary"]["files_checked"], 1);
    }

    #[test]
    fn test_format_summary_no_issues() {
        let mut buf = Vec::new();
        format_summary(&[], 5, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No issues found in 5 file(s)"));
    }
}
