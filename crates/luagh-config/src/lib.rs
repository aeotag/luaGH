//! # luagh-config
//!
//! Configuration file parsing and resolution for LuaGH.
//! Supports `luagh.toml` and `.luagh.toml` with hierarchical lookup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use luagh_core::LuaVersion;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Config Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error reading config: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid regex in naming config for {field}: {message}")]
    InvalidRegex { field: String, message: String },
}

// ---------------------------------------------------------------------------
// Top-level Config
// ---------------------------------------------------------------------------

/// Root configuration structure corresponding to `luagh.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Lua standard version.
    pub std: LuaVersion,

    /// File include/exclude patterns.
    pub files: FilesConfig,

    /// Known global variables.
    pub globals: GlobalsConfig,

    /// Per-rule configuration overrides.
    pub rules: HashMap<String, RuleOverride>,

    /// Naming convention patterns.
    pub naming: NamingConfig,

    /// Per-path override sections.
    #[serde(default)]
    pub overrides: Vec<PathOverride>,
}

// ---------------------------------------------------------------------------
// Sub-configs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FilesConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            include: vec!["**/*.lua".to_string()],
            exclude: vec![
                "vendor/**".to_string(),
                "node_modules/**".to_string(),
                ".luarocks/**".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct GlobalsConfig {
    /// Read-write globals.
    pub rw: Vec<String>,
    /// Read-only globals (writes produce a warning).
    pub ro: Vec<String>,
}

impl GlobalsConfig {
    /// All configured globals (both read-write and read-only).
    pub fn all_globals(&self) -> impl Iterator<Item = &str> {
        self.rw.iter().chain(self.ro.iter()).map(|s| s.as_str())
    }

    /// Check if a name is a known global.
    pub fn is_known(&self, name: &str) -> bool {
        self.rw.iter().any(|g| g == name) || self.ro.iter().any(|g| g == name)
    }

    /// Check if a global is read-only.
    pub fn is_read_only(&self, name: &str) -> bool {
        self.ro.iter().any(|g| g == name)
    }
}

// ---------------------------------------------------------------------------
// Rule Override
// ---------------------------------------------------------------------------

/// A rule can be set to off, or overridden with severity and/or custom settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleOverride {
    /// Simple string: "off", "warning", "error", etc.
    Simple(String),
    /// Table with severity and custom fields.
    Detailed(DetailedRuleOverride),
}

impl RuleOverride {
    /// Returns `true` if the rule is disabled.
    pub fn is_off(&self) -> bool {
        match self {
            RuleOverride::Simple(s) => s == "off" || s == "false",
            RuleOverride::Detailed(d) => d.severity.as_deref() == Some("off"),
        }
    }

    /// Returns the overridden severity, if any.
    pub fn severity(&self) -> Option<&str> {
        match self {
            RuleOverride::Simple(s) if s != "off" && s != "false" => Some(s.as_str()),
            RuleOverride::Detailed(d) => d.severity.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailedRuleOverride {
    pub severity: Option<String>,
    /// Rule-specific settings (e.g., max line length).
    #[serde(flatten)]
    pub settings: HashMap<String, toml::Value>,
}

// ---------------------------------------------------------------------------
// Naming Config
// ---------------------------------------------------------------------------

/// Naming convention patterns for each symbol kind.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NamingConfig {
    pub local_variable: Option<String>,
    pub global_variable: Option<String>,
    pub function: Option<String>,
    pub method: Option<String>,
    pub constant: Option<String>,
    pub parameter: Option<String>,
    pub ignore_names: Vec<String>,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            local_variable: Some(r"^[a-z_][a-z0-9_]*$".to_string()),
            global_variable: Some(r"^[a-z_][a-z0-9_]*$".to_string()),
            function: Some(r"^[A-Z][A-Za-z0-9]*$".to_string()),
            method: Some(r"^[A-Z][A-Za-z0-9]*$".to_string()),
            constant: Some(r"^[A-Z][A-Z0-9_]*$".to_string()),
            parameter: None, // No default naming requirement for parameters
            ignore_names: vec![
                "_".to_string(),
                "_G".to_string(),
                "_ENV".to_string(),
                "_VERSION".to_string(),
                "self".to_string(),
                "__index".to_string(),
                "__newindex".to_string(),
                "__call".to_string(),
                "__tostring".to_string(),
                "__add".to_string(),
                "__sub".to_string(),
                "__mul".to_string(),
                "__div".to_string(),
                "__mod".to_string(),
                "__pow".to_string(),
                "__unm".to_string(),
                "__concat".to_string(),
                "__len".to_string(),
                "__eq".to_string(),
                "__lt".to_string(),
                "__le".to_string(),
                "__gc".to_string(),
                "__mode".to_string(),
                "__metatable".to_string(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Path Override
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct PathOverride {
    pub paths: Vec<String>,
    #[serde(default)]
    pub rules: HashMap<String, RuleOverride>,
    #[serde(default)]
    pub naming: Option<NamingConfig>,
}

// ---------------------------------------------------------------------------
// Config Loading
// ---------------------------------------------------------------------------

/// Config file names to search for, in order of priority.
const CONFIG_FILE_NAMES: &[&str] = &["luagh.toml", ".luagh.toml"];

/// Resolve config file by searching from `start_dir` upward.
pub fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        for name in CONFIG_FILE_NAMES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Load config from a specific file path.
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// Load config from the default location, or return defaults.
pub fn load_config_or_default(start_dir: &Path) -> Config {
    match find_config_file(start_dir) {
        Some(path) => match load_config(&path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "warning: failed to load config from {}: {e}",
                    path.display()
                );
                Config::default()
            }
        },
        None => Config::default(),
    }
}

/// Generate a default config file as a TOML string.
pub fn generate_default_config() -> String {
    r#"# luagh.toml — LuaGH configuration
# See: https://github.com/yannic/luaGH/blob/main/ARCHITECTURE.md

# Lua standard version: lua51, lua52, lua53, lua54, luajit, luau
std = "lua54"

# File selection
[files]
include = ["**/*.lua"]
exclude = ["vendor/**", "node_modules/**", ".luarocks/**"]

# Known globals (in addition to the standard library)
[globals]
# Read-write globals
rw = []
# Read-only globals (writing to these produces a warning)
ro = []

# Rule configuration
# Set a rule to "off" to disable, or override its severity.
[rules]
# "lint.shadowing" = "off"
# "lint.unused_local" = "warning"
# "naming.function_case" = "error"
# "style.line_length" = { severity = "warning", max = 120 }

# Naming conventions (regex patterns per symbol kind)
[naming]
local_variable = "^[a-z_][a-z0-9_]*$"
global_variable = "^[a-z_][a-z0-9_]*$"
function = "^[A-Z][A-Za-z0-9]*$"
method = "^[A-Z][A-Za-z0-9]*$"
constant = "^[A-Z][A-Z0-9_]*$"
ignore_names = [
    "_", "_G", "_ENV", "_VERSION", "self",
    "__index", "__newindex", "__call", "__tostring",
    "__add", "__sub", "__mul", "__div", "__mod",
    "__pow", "__unm", "__concat", "__len", "__eq",
    "__lt", "__le", "__gc", "__mode", "__metatable",
]

# Per-path overrides
# [[overrides]]
# paths = ["tests/**"]
# [overrides.rules]
# "lint.undefined_global" = "off"
# "naming.function_case" = "off"
"#
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.std, LuaVersion::Lua54);
        assert_eq!(config.files.include, vec!["**/*.lua"]);
        assert!(config.naming.local_variable.is_some());
    }

    #[test]
    fn test_parse_config_toml() {
        let toml_str = r#"
            std = "lua51"

            [files]
            include = ["src/**/*.lua"]
            exclude = ["vendor/**"]

            [globals]
            rw = ["MY_GLOBAL"]
            ro = ["love"]

            [rules]
            "lint.shadowing" = "off"
            "lint.unused_local" = "warning"

            [naming]
            local_variable = "^[a-z_][a-z0-9_]*$"
            function = "^[A-Z][A-Za-z0-9]*$"
            ignore_names = ["_", "self"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.std, LuaVersion::Lua51);
        assert_eq!(config.globals.rw, vec!["MY_GLOBAL"]);
        assert!(config.rules.get("lint.shadowing").unwrap().is_off());
        assert_eq!(
            config.rules.get("lint.unused_local").unwrap().severity(),
            Some("warning")
        );
    }

    #[test]
    fn test_generate_default_config_parses() {
        let generated = generate_default_config();
        let _config: Config = toml::from_str(&generated).unwrap();
    }

    #[test]
    fn test_globals_config() {
        let globals = GlobalsConfig {
            rw: vec!["MY_GLOBAL".to_string()],
            ro: vec!["love".to_string()],
        };
        assert!(globals.is_known("MY_GLOBAL"));
        assert!(globals.is_known("love"));
        assert!(!globals.is_known("unknown"));
        assert!(!globals.is_read_only("MY_GLOBAL"));
        assert!(globals.is_read_only("love"));
    }
}
