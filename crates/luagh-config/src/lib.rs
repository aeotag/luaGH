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

    /// Named regex patterns that can be referenced by name in `[naming]`.
    pub regex: HashMap<String, String>,

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
///
/// Values can be either raw regex strings (e.g. `"^[a-z_][a-z0-9_]*$"`) or
/// names referencing a pattern defined in the `[regex]` table (e.g. `"snake_case"`).
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

impl NamingConfig {
    /// Resolve named pattern references through the `[regex]` table.
    ///
    /// If a naming field's value matches a key in `patterns`, it is replaced
    /// with the corresponding regex string. Values that already look like
    /// regex (or don't match any key) are left as-is.
    pub fn resolve(&mut self, patterns: &HashMap<String, String>) {
        fn resolve_field(field: &mut Option<String>, patterns: &HashMap<String, String>) {
            if let Some(val) = field.as_ref()
                && let Some(regex) = patterns.get(val.as_str())
            {
                *field = Some(regex.clone());
            }
        }
        resolve_field(&mut self.local_variable, patterns);
        resolve_field(&mut self.global_variable, patterns);
        resolve_field(&mut self.function, patterns);
        resolve_field(&mut self.method, patterns);
        resolve_field(&mut self.constant, patterns);
        resolve_field(&mut self.parameter, patterns);
    }
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
    let mut config: Config = toml::from_str(&content)?;
    // Resolve named regex references in naming config
    config.naming.resolve(&config.regex);
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

[files]
include = ["**/*.lua"]
exclude = ["vendor/**", "node_modules/**", ".luarocks/**"]

[globals]
rw = []
ro = []

# Named regex patterns — reference these by name in [naming]
[regex]
snake_case           = "^[a-z_][a-z0-9_]*$"
pascal_case          = "^[A-Z][A-Za-z0-9]*$"
screaming_snake_case = "^[A-Z][A-Z0-9_]*$"
camel_case           = "^[a-z][a-zA-Z0-9]*$"

[rules]
# "lint.shadowing" = "off"
# "lint.unused_local" = "warning"
# "naming.function_case" = "error"

# Naming conventions — use pattern names from [regex] or raw regex strings
[naming]
local_variable  = "snake_case"
global_variable = "pascal_case"
function        = "pascal_case"
method          = "snake_case"
constant        = "screaming_snake_case"
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

    #[test]
    fn test_regex_named_patterns_resolve() {
        let toml_str = r#"
            std = "lua54"

            [regex]
            snake_case  = "^[a-z_][a-z0-9_]*$"
            pascal_case = "^[A-Z][A-Za-z0-9]*$"

            [naming]
            local_variable  = "snake_case"
            global_variable = "pascal_case"
            function        = "pascal_case"
        "#;

        let mut config: Config = toml::from_str(toml_str).unwrap();
        config.naming.resolve(&config.regex);

        assert_eq!(
            config.naming.local_variable.as_deref(),
            Some("^[a-z_][a-z0-9_]*$")
        );
        assert_eq!(
            config.naming.global_variable.as_deref(),
            Some("^[A-Z][A-Za-z0-9]*$")
        );
        assert_eq!(
            config.naming.function.as_deref(),
            Some("^[A-Z][A-Za-z0-9]*$")
        );
    }

    #[test]
    fn test_regex_raw_passthrough() {
        let toml_str = r#"
            std = "lua54"

            [naming]
            local_variable = "^[a-z_][a-z0-9_]*$"
        "#;

        let mut config: Config = toml::from_str(toml_str).unwrap();
        config.naming.resolve(&config.regex);

        // Raw regex should pass through unchanged when no [regex] table matches
        assert_eq!(
            config.naming.local_variable.as_deref(),
            Some("^[a-z_][a-z0-9_]*$")
        );
    }
}
