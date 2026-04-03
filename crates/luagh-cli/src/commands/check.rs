//! The `luagh check` command — the main analysis pipeline.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use walkdir::WalkDir;

use luagh_config::{self, Config};
use luagh_core::{Diagnostic, LuaVersion, Severity};
use luagh_parser;
use luagh_rules::{RuleContext, RuleRegistry};
use luagh_sema;

use crate::Cli;
use crate::output;

/// Run all checks on the given paths.
pub fn run(paths: &[PathBuf], cli: &Cli) -> Result<bool, Box<dyn std::error::Error>> {
    let config = load_config(cli)?;
    let lua_version = resolve_lua_version(cli, &config);
    let registry = RuleRegistry::builtin();
    let files = discover_files(paths, &config)?;

    let diagnostics = analyze_files(&files, &config, lua_version, &registry);
    let fail_level: Severity = cli.fail_on.clone().into();
    let has_failures = diagnostics.iter().any(|d| d.severity >= fail_level);

    output::write_output(
        &diagnostics,
        files.len(),
        cli.format.clone().into(),
        cli.quiet,
    )?;

    Ok(has_failures)
}

/// Run checks for a specific category (e.g., "lint" or "naming").
pub fn run_category(
    paths: &[PathBuf],
    cli: &Cli,
    category: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let config = load_config(cli)?;
    let lua_version = resolve_lua_version(cli, &config);
    let registry = RuleRegistry::builtin();
    let files = discover_files(paths, &config)?;

    let diagnostics = analyze_files_category(&files, &config, lua_version, &registry, category);
    let fail_level: Severity = cli.fail_on.clone().into();
    let has_failures = diagnostics.iter().any(|d| d.severity >= fail_level);

    output::write_output(
        &diagnostics,
        files.len(),
        cli.format.clone().into(),
        cli.quiet,
    )?;

    Ok(has_failures)
}

/// Syntax-only check (parse only, no semantic analysis).
pub fn run_syntax_only(paths: &[PathBuf], cli: &Cli) -> Result<bool, Box<dyn std::error::Error>> {
    let config = load_config(cli)?;
    let files = discover_files(paths, &config)?;

    let diagnostics: Vec<Diagnostic> = files
        .par_iter()
        .flat_map(|file| match luagh_parser::parse_file(file) {
            Ok(_parsed) => Vec::new(),
            Err(luagh_parser::ParseError::Parse { diagnostics, .. }) => diagnostics,
            Err(luagh_parser::ParseError::Io { path, source }) => {
                vec![Diagnostic::new(
                    "syntax.io_error",
                    Severity::Error,
                    format!("cannot read file: {source}"),
                    path,
                    luagh_core::Span::default(),
                )]
            }
        })
        .collect();

    let fail_level: Severity = cli.fail_on.clone().into();
    let has_failures = diagnostics.iter().any(|d| d.severity >= fail_level);

    output::write_output(
        &diagnostics,
        files.len(),
        cli.format.clone().into(),
        cli.quiet,
    )?;

    Ok(has_failures)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn load_config(cli: &Cli) -> Result<Config, Box<dyn std::error::Error>> {
    match &cli.config {
        Some(path) => Ok(luagh_config::load_config(path)?),
        None => {
            let cwd = std::env::current_dir()?;
            Ok(luagh_config::load_config_or_default(&cwd))
        }
    }
}

fn resolve_lua_version(cli: &Cli, config: &Config) -> LuaVersion {
    if let Some(ref std_str) = cli.std {
        std_str.parse().unwrap_or(config.std)
    } else {
        config.std
    }
}

fn discover_files(
    paths: &[PathBuf],
    config: &Config,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let include_set = build_glob_set(&config.files.include)?;
    let exclude_set = build_glob_set(&config.files.exclude)?;

    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() && should_include(entry_path, &include_set, &exclude_set) {
                    files.push(entry_path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet, Box<dyn std::error::Error>> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn should_include(path: &Path, include_set: &GlobSet, exclude_set: &GlobSet) -> bool {
    let path_str = path.to_string_lossy();
    // Normalize backslashes to forward slashes for glob matching
    let normalized = path_str.replace('\\', "/");
    include_set.is_match(&normalized) && !exclude_set.is_match(&normalized)
}

fn analyze_files(
    files: &[PathBuf],
    config: &Config,
    lua_version: LuaVersion,
    registry: &RuleRegistry,
) -> Vec<Diagnostic> {
    files
        .par_iter()
        .flat_map(|file| analyze_single_file(file, config, lua_version, registry))
        .collect()
}

fn analyze_files_category(
    files: &[PathBuf],
    config: &Config,
    lua_version: LuaVersion,
    registry: &RuleRegistry,
    category: &str,
) -> Vec<Diagnostic> {
    files
        .par_iter()
        .flat_map(|file| {
            analyze_single_file_category(file, config, lua_version, registry, category)
        })
        .collect()
}

fn analyze_single_file(
    file: &Path,
    config: &Config,
    lua_version: LuaVersion,
    registry: &RuleRegistry,
) -> Vec<Diagnostic> {
    // Parse
    let parsed = match luagh_parser::parse_file(file) {
        Ok(p) => p,
        Err(luagh_parser::ParseError::Parse { diagnostics, .. }) => return diagnostics,
        Err(luagh_parser::ParseError::Io { path, source }) => {
            return vec![Diagnostic::new(
                "syntax.io_error",
                Severity::Error,
                format!("cannot read file: {source}"),
                path,
                luagh_core::Span::default(),
            )];
        }
    };

    // Semantic analysis
    let sema = luagh_sema::analyze(&parsed, lua_version);

    // Build rule context
    let ctx = RuleContext {
        file_path: &parsed.path,
        source: &parsed.source,
        ast: &parsed.ast,
        symbols: &sema.symbols,
        scopes: &sema.scopes,
        config,
        lua_version,
        line_index: &parsed.line_index,
    };

    // Run all rules
    let mut diagnostics = sema.diagnostics;
    diagnostics.extend(parsed.parse_errors);
    diagnostics.extend(registry.check_all(&ctx));
    diagnostics
}

fn analyze_single_file_category(
    file: &Path,
    config: &Config,
    lua_version: LuaVersion,
    registry: &RuleRegistry,
    category: &str,
) -> Vec<Diagnostic> {
    let parsed = match luagh_parser::parse_file(file) {
        Ok(p) => p,
        Err(luagh_parser::ParseError::Parse { diagnostics, .. }) => return diagnostics,
        Err(luagh_parser::ParseError::Io { path, source }) => {
            return vec![Diagnostic::new(
                "syntax.io_error",
                Severity::Error,
                format!("cannot read file: {source}"),
                path,
                luagh_core::Span::default(),
            )];
        }
    };

    let sema = luagh_sema::analyze(&parsed, lua_version);

    let ctx = RuleContext {
        file_path: &parsed.path,
        source: &parsed.source,
        ast: &parsed.ast,
        symbols: &sema.symbols,
        scopes: &sema.scopes,
        config,
        lua_version,
        line_index: &parsed.line_index,
    };

    registry.check_category(&ctx, category)
}
