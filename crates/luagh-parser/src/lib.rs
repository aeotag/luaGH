//! # luagh-parser
//!
//! Parser wrapper around `full_moon`. Provides a LuaGH-specific interface
//! that isolates the rest of the codebase from the parser implementation.

use std::path::{Path, PathBuf};

use full_moon::ast::Ast;
use luagh_core::{Diagnostic, LineIndex, Position, Severity, Span};

/// A parsed Lua source file, bundling the AST with metadata.
#[derive(Debug)]
pub struct ParsedFile {
    /// File path (for diagnostics).
    pub path: PathBuf,
    /// Original source text.
    pub source: String,
    /// Parsed AST (present even if there were recoverable parse warnings).
    pub ast: Ast,
    /// Parse errors converted to LuaGH diagnostics.
    pub parse_errors: Vec<Diagnostic>,
    /// Line index for efficient offset-to-position lookups.
    pub line_index: LineIndex,
}

/// Errors that can occur during parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("parse failed for {path}")]
    Parse {
        path: PathBuf,
        diagnostics: Vec<Diagnostic>,
    },
}

/// Parse a Lua source file from disk.
pub fn parse_file(path: &Path) -> Result<ParsedFile, ParseError> {
    let source = std::fs::read_to_string(path).map_err(|e| ParseError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_source(&source, path)
}

/// Parse Lua source text given as a string.
pub fn parse_source(source: &str, path: &Path) -> Result<ParsedFile, ParseError> {
    let line_index = LineIndex::new(source);

    match full_moon::parse(source) {
        Ok(ast) => Ok(ParsedFile {
            path: path.to_path_buf(),
            source: source.to_string(),
            ast,
            parse_errors: Vec::new(),
            line_index,
        }),
        Err(errors) => {
            let diagnostics: Vec<Diagnostic> = errors
                .iter()
                .map(|err| convert_parse_error(err, path, &line_index))
                .collect();

            Err(ParseError::Parse {
                path: path.to_path_buf(),
                diagnostics,
            })
        }
    }
}

/// Convert a full_moon parse error into a LuaGH diagnostic.
fn convert_parse_error(
    error: &full_moon::Error,
    path: &Path,
    _line_index: &LineIndex,
) -> Diagnostic {
    let message = error.to_string();

    // Extract position from the error if available
    let span = match error {
        full_moon::Error::AstError(ast_err) => {
            let token = ast_err.token();
            let start_pos = token.start_position();
            let end_pos = token.end_position();
            Span::new(
                Position::new(
                    start_pos.line() as u32 - 1,
                    start_pos.character() as u32 - 1,
                    0,
                ),
                Position::new(end_pos.line() as u32 - 1, end_pos.character() as u32 - 1, 0),
            )
        }
        full_moon::Error::TokenizerError(tok_err) => {
            let pos = tok_err.position();
            let p = Position::new(pos.line() as u32 - 1, pos.character() as u32 - 1, 0);
            Span::single(p)
        }
    };

    Diagnostic::new("syntax.parse_error", Severity::Error, message, path, span)
}

/// Extract the source excerpt (the line of source code) for a given span.
pub fn source_excerpt(source: &str, line_index: &LineIndex, span: &Span) -> Option<String> {
    line_index
        .line_text(source, span.start.line)
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_valid_lua() {
        let source = "local x = 1\nprint(x)\n";
        let path = PathBuf::from("test.lua");
        let result = parse_source(source, &path);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.parse_errors.is_empty());
    }

    #[test]
    fn test_parse_invalid_lua() {
        let source = "local = = =";
        let path = PathBuf::from("bad.lua");
        let result = parse_source(source, &path);
        assert!(result.is_err());
        if let Err(ParseError::Parse { diagnostics, .. }) = result {
            assert!(!diagnostics.is_empty());
            assert_eq!(diagnostics[0].rule_id, "syntax.parse_error");
        }
    }

    #[test]
    fn test_source_excerpt() {
        let source = "local x = 1\nprint(x)\nreturn x\n";
        let idx = LineIndex::new(source);
        let span = Span::new(Position::new(1, 0, 12), Position::new(1, 8, 20));
        let excerpt = source_excerpt(source, &idx, &span);
        assert_eq!(excerpt, Some("print(x)".to_string()));
    }
}
