//! The `luagh init` command — generates a default configuration file.

use std::path::Path;

use luagh_config::generate_default_config;

pub fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let config_path = Path::new("luagh.toml");

    if config_path.exists() {
        eprintln!("luagh.toml already exists. Remove it first to regenerate.");
        return Err("config file already exists".into());
    }

    let content = generate_default_config();
    std::fs::write(config_path, content)?;
    println!("Created luagh.toml with default configuration.");
    println!("Edit it to customize rules, naming conventions, and globals.");

    Ok(false)
}
