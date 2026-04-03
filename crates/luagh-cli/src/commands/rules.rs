//! The `luagh rules` command — lists all available rules.

use luagh_rules::RuleRegistry;

pub fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let registry = RuleRegistry::builtin();

    println!("{:<35} {:<10} {}", "RULE ID", "SEVERITY", "DESCRIPTION");
    println!("{}", "-".repeat(80));

    for rule in registry.iter() {
        println!(
            "{:<35} {:<10} {}",
            rule.id(),
            rule.default_severity(),
            rule.description()
        );
    }

    println!();
    println!("Total: {} rules", registry.len());
    println!();
    println!("Use `luagh explain <rule-id>` for detailed information about a rule.");

    Ok(false)
}
