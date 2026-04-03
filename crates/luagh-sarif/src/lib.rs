//! # luagh-sarif
//!
//! SARIF v2.1.0 output for GitHub code scanning integration.
//! Converts LuaGH diagnostics into the SARIF JSON format.

use std::io::Write;

use luagh_core::{Diagnostic, Severity};
use serde::Serialize;

// ---------------------------------------------------------------------------
// SARIF Schema Types (v2.1.0)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDriver {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    pub rules: Vec<SarifRuleDescriptor>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRuleDescriptor {
    pub id: String,
    pub short_description: SarifMessage,
    pub default_configuration: SarifDefaultConfiguration,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDefaultConfiguration {
    pub level: String,
}

#[derive(Serialize)]
pub struct SarifResult {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    pub region: SarifRegion,
}

#[derive(Serialize)]
pub struct SarifArtifactLocation {
    pub uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

/// Convert LuaGH diagnostics to a SARIF log.
pub fn to_sarif(diagnostics: &[Diagnostic], tool_version: &str) -> SarifLog {
    // Collect unique rule IDs for the rules array
    let mut rule_ids: Vec<String> = diagnostics
        .iter()
        .map(|d| d.rule_id.clone())
        .collect();
    rule_ids.sort();
    rule_ids.dedup();

    let rules: Vec<SarifRuleDescriptor> = rule_ids
        .iter()
        .map(|id| {
            // Find the first diagnostic with this rule ID for metadata
            let sample = diagnostics.iter().find(|d| &d.rule_id == id).unwrap();
            SarifRuleDescriptor {
                id: id.clone(),
                short_description: SarifMessage {
                    text: sample.message.clone(),
                },
                default_configuration: SarifDefaultConfiguration {
                    level: sample.severity.sarif_level().to_string(),
                },
            }
        })
        .collect();

    let results: Vec<SarifResult> = diagnostics
        .iter()
        .map(|d| SarifResult {
            rule_id: d.rule_id.clone(),
            level: d.severity.sarif_level().to_string(),
            message: SarifMessage {
                text: d.message.clone(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: d.file_path.display().to_string().replace('\\', "/"),
                    },
                    region: SarifRegion {
                        start_line: d.span.start.display_line(),
                        start_column: d.span.start.display_column(),
                        end_line: d.span.end.display_line(),
                        end_column: d.span.end.display_column(),
                    },
                },
            }],
        })
        .collect();

    SarifLog {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "LuaGH".to_string(),
                    version: tool_version.to_string(),
                    information_uri: Some("https://github.com/yannic/luaGH".to_string()),
                    rules,
                },
            },
            results,
        }],
    }
}

/// Write SARIF output to a writer.
pub fn format_sarif(
    diagnostics: &[Diagnostic],
    tool_version: &str,
    writer: &mut dyn Write,
) -> std::io::Result<()> {
    let sarif = to_sarif(diagnostics, tool_version);
    let json = serde_json::to_string_pretty(&sarif)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
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

    #[test]
    fn test_sarif_generation() {
        let diags = vec![Diagnostic::new(
            "lint.unused_local",
            Severity::Warning,
            "unused local variable `x`",
            PathBuf::from("src/test.lua"),
            Span::new(Position::new(4, 6, 40), Position::new(4, 7, 41)),
        )];

        let sarif = to_sarif(&diags, "0.1.0");
        assert_eq!(sarif.version, "2.1.0");
        assert_eq!(sarif.runs.len(), 1);
        assert_eq!(sarif.runs[0].results.len(), 1);
        assert_eq!(sarif.runs[0].results[0].rule_id, "lint.unused_local");
        assert_eq!(sarif.runs[0].results[0].level, "warning");
        assert_eq!(sarif.runs[0].tool.driver.rules.len(), 1);
    }

    #[test]
    fn test_sarif_json_output() {
        let diags = vec![Diagnostic::new(
            "lint.undefined_global",
            Severity::Error,
            "use of undefined global `prnt`",
            PathBuf::from("main.lua"),
            Span::new(Position::new(2, 0, 20), Position::new(2, 4, 24)),
        )];

        let mut buf = Vec::new();
        format_sarif(&diags, "0.1.0", &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(
            parsed["runs"][0]["results"][0]["ruleId"],
            "lint.undefined_global"
        );
    }
}
