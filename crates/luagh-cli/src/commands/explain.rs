//! The `luagh explain` command — shows detailed help for a rule.

use luagh_rules::RuleRegistry;

pub fn run(rule_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let registry = RuleRegistry::builtin();

    match registry.get(rule_id) {
        Some(rule) => {
            println!("Rule: {}", rule.id());
            println!("Name: {}", rule.name());
            println!("Category: {}", rule.category());
            println!("Default severity: {}", rule.default_severity());
            println!();
            println!("Description:");
            println!("  {}", rule.description());
            println!();

            let help = rule.help();
            if !help.is_empty() {
                println!("Details:");
                for line in help.lines() {
                    println!("  {line}");
                }
            }

            Ok(false)
        }
        None => {
            eprintln!("unknown rule: {rule_id}");
            eprintln!();
            eprintln!("Available rules:");
            for rule in registry.iter() {
                eprintln!("  {}", rule.id());
            }
            Err(format!("unknown rule: {rule_id}").into())
        }
    }
}
