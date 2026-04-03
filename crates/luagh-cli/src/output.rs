//! Output formatting dispatcher.

use luagh_core::{Diagnostic, OutputFormat};

/// Write diagnostics to stdout in the requested format.
pub fn write_output(
    diagnostics: &[Diagnostic],
    files_checked: usize,
    format: OutputFormat,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if quiet && diagnostics.is_empty() {
        return Ok(());
    }

    let stdout = std::io::stdout();
    let mut writer = stdout.lock();

    match format {
        OutputFormat::Text => {
            luagh_diagnostics::format_text(diagnostics, &mut writer)?;
            if !quiet {
                luagh_diagnostics::format_summary(diagnostics, files_checked, &mut writer)?;
            }
        }
        OutputFormat::Json => {
            luagh_diagnostics::format_json(diagnostics, files_checked, &mut writer)?;
        }
        OutputFormat::Sarif => {
            let version = env!("CARGO_PKG_VERSION");
            luagh_sarif::format_sarif(diagnostics, version, &mut writer)?;
        }
    }

    Ok(())
}
