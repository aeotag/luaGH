# LuaGH — Lua Guard Hub

A fast, extensible static analysis and linting tool for **Lua** and **Luau**, written in Rust.

LuaGH catches bugs, enforces naming conventions, and integrates with GitHub Actions via SARIF for inline code scanning results.

## Features

- **Syntax validation** — Parse Lua/Luau and report precise syntax errors
- **Lint rules** — Detect unused locals, undefined globals, variable shadowing
- **Naming conventions** — Enforce snake\_case, PascalCase, UPPER\_CASE per symbol kind
- **Configurable** — TOML-based config with per-path overrides
- **Multiple output formats** — Human-readable text, JSON, and SARIF v2.1.0
- **Parallel analysis** — File-level parallelism via rayon
- **GitHub Actions ready** — SARIF upload for inline PR annotations

## Quick Start

```bash
# Build from source
cargo build --release

# Run all checks on current directory
luagh check .

# Lint only
luagh lint src/

# Syntax check only
luagh syntax game.lua

# Naming convention check
luagh naming lib/

# Generate default config
luagh init
```

## Configuration

Create a `luagh.toml` (or `.luagh.toml`) in your project root:

```toml
std = "lua54"

[files]
include = ["**/*.lua", "**/*.luau"]
exclude = ["**/vendor/**"]

[rules]
"lint.unused_local"     = "warning"
"lint.undefined_global" = "error"
"lint.shadowing"        = "info"

[naming]
local_variable = "^[a-z_][a-zA-Z0-9_]*$"
global_variable = "^[A-Z_][A-Z0-9_]*$"
function = "^[a-z_][a-zA-Z0-9_]*$"
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `luagh check [paths]` | Run all checks (lint + naming + semantic) |
| `luagh lint [paths]` | Run lint checks only |
| `luagh syntax [paths]` | Parse validation only |
| `luagh naming [paths]` | Naming convention checks only |
| `luagh explain <rule>` | Show detailed help for a rule |
| `luagh rules` | List all available rules |
| `luagh init` | Generate default `luagh.toml` |

**Global flags:** `--format text|json|sarif`, `--fail-on hint|info|warning|error`, `--config <path>`, `--std <version>`, `--threads <n>`, `--quiet`

## Built-in Rules

| Rule ID | Category | Default | Description |
|---------|----------|---------|-------------|
| `lint.unused_local` | lint | warning | Detect unused local variables |
| `lint.undefined_global` | lint | error | Detect use of undefined globals |
| `lint.shadowing` | lint | info | Detect variable shadowing |
| `naming.local_variable_case` | naming | warning | Enforce local variable naming |
| `naming.global_variable_case` | naming | warning | Enforce global variable naming |
| `naming.function_case` | naming | warning | Enforce function naming |
| `naming.method_case` | naming | warning | Enforce method naming |

## GitHub Actions

```yaml
- name: Run LuaGH
  run: luagh check --format sarif . > results.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

See [.github/workflows/luagh-example.yml](.github/workflows/luagh-example.yml) for a complete workflow.

## Architecture

LuaGH is organized as an 8-crate Cargo workspace:

| Crate | Purpose |
|-------|---------|
| `luagh-cli` | CLI entry point (clap) |
| `luagh-core` | Shared types (spans, severity, diagnostics) |
| `luagh-parser` | full\_moon parser wrapper |
| `luagh-sema` | Semantic analysis (scopes, symbols) |
| `luagh-rules` | Rule engine and built-in rules |
| `luagh-config` | TOML configuration parsing |
| `luagh-diagnostics` | Text/JSON output formatting |
| `luagh-sarif` | SARIF v2.1.0 output |

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full design document.

## License

MIT — see [LICENSE](LICENSE)