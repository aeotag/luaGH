//! LuaGH CLI — entry point for the `luagh` command.

mod commands;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

/// LuaGH — Lua Guard Hub: fast static analysis for Lua and Luau.
#[derive(Parser)]
#[command(name = "luagh", version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// Path to config file
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub(crate) format: OutputFormatArg,

    /// Exit non-zero if any diagnostic is at or above this level
    #[arg(long, global = true, default_value = "error")]
    pub(crate) fail_on: SeverityArg,

    /// Disable caching
    #[arg(long, global = true)]
    pub(crate) no_cache: bool,

    /// Show timing information
    #[arg(long, global = true)]
    pub(crate) timings: bool,

    /// Number of parallel threads (0 = auto)
    #[arg(long, global = true, default_value = "0")]
    pub(crate) threads: usize,

    /// Additional include glob patterns
    #[arg(long, global = true)]
    pub(crate) include: Vec<String>,

    /// Additional exclude glob patterns
    #[arg(long, global = true)]
    pub(crate) exclude: Vec<String>,

    /// Lua standard version (overrides config)
    #[arg(long, global = true)]
    pub(crate) std: Option<String>,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all checks (lint + naming + semantic)
    Check {
        /// Files or directories to check
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Run lint checks only
    Lint {
        /// Files or directories to check
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Check syntax only (fast parse validation)
    Syntax {
        /// Files or directories to check
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Run naming convention checks only
    Naming {
        /// Files or directories to check
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Show detailed help for a rule
    Explain {
        /// Rule ID (e.g., lint.unused_local)
        rule_id: String,
    },
    /// List all available rules
    Rules,
    /// Generate a default luagh.toml configuration file
    Init,
    /// Print version information
    Version,
}

#[derive(Clone, ValueEnum)]
pub(crate) enum OutputFormatArg {
    Text,
    Json,
    Sarif,
}

impl From<OutputFormatArg> for luagh_core::OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Text => luagh_core::OutputFormat::Text,
            OutputFormatArg::Json => luagh_core::OutputFormat::Json,
            OutputFormatArg::Sarif => luagh_core::OutputFormat::Sarif,
        }
    }
}

#[derive(Clone, ValueEnum)]
pub(crate) enum SeverityArg {
    Hint,
    Info,
    Warning,
    Error,
}

impl From<SeverityArg> for luagh_core::Severity {
    fn from(arg: SeverityArg) -> Self {
        match arg {
            SeverityArg::Hint => luagh_core::Severity::Hint,
            SeverityArg::Info => luagh_core::Severity::Info,
            SeverityArg::Warning => luagh_core::Severity::Warning,
            SeverityArg::Error => luagh_core::Severity::Error,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Configure thread pool
    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()
            .ok();
    }

    let start = std::time::Instant::now();

    let result = match cli.command {
        Commands::Check { ref paths } => commands::check::run(paths, &cli),
        Commands::Lint { ref paths } => commands::check::run_category(paths, &cli, "lint"),
        Commands::Syntax { ref paths } => commands::check::run_syntax_only(paths, &cli),
        Commands::Naming { ref paths } => commands::check::run_category(paths, &cli, "naming"),
        Commands::Explain { ref rule_id } => commands::explain::run(rule_id),
        Commands::Rules => commands::rules::run(),
        Commands::Init => commands::init::run(),
        Commands::Version => {
            println!("luagh {}", env!("CARGO_PKG_VERSION"));
            Ok(false)
        }
    };

    if cli.timings {
        let elapsed = start.elapsed();
        eprintln!("Elapsed: {elapsed:.3?}");
    }

    match result {
        Ok(has_failures) => {
            if has_failures {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
