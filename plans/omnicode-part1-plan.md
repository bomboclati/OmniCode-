# OmniCode - Part 1 Implementation Plan

## Overview
Build the foundational Rust project structure for the OmniCode AI coding agent CLI.

## Files to Create

1. **`omnicode/Cargo.toml`** - Package manifest with all dependencies
2. **`omnicode/build.rs`** - Build script to embed web UI assets via rust-embed
3. **`omnicode/src/main.rs`** - Entry point with CLI dispatch, tracing, and subcommand routing
4. **`omnicode/src/cli.rs`** - Clap derive-based CLI argument parsing with all subcommands
5. **`omnicode/src/config.rs`** - Configuration management (profiles, providers, encryption-at-rest)
6. **`omnicode/src/crypto.rs`** - AES-256-GCM encryption/decryption with machine-derived key

## Implementation Order

| Step | File | Description |
|------|------|-------------|
| 1 | `Cargo.toml` | All dependencies, features, release profile |
| 2 | `build.rs` | RustEmbed build script |
| 3 | `src/crypto.rs` | No dependencies on other modules, foundational |
| 4 | `src/config.rs` | Depends on crypto, implements config load/save |
| 5 | `src/cli.rs` | Independent, pure CLI parsing |
| 6 | `src/main.rs` | Depends on cli + config, ties everything together |

## Dependency Graph

- `main.rs` → `cli.rs`, `config.rs`
- `config.rs` → `crypto.rs`
- `crypto.rs` → standalone
- `cli.rs` → standalone
- `build.rs` → standalone

## Key Design Decisions

1. **Encryption**: AES-256-GCM with random 12-byte nonce, machine-bound key derived from UUID + hostname + salt via SHA-256
2. **CLI**: Use `clap` derive macros for type-safe argument parsing
3. **Config**: TOML format stored at `~/.omnicode/config.toml`, API keys encrypted before save
4. **Subcommands**: 20+ subcommands all dispatched from `main.rs` with async handlers
5. **Cross-platform**: Windows uses registry for machine ID, Unix uses `/etc/machine-id`

## Subcommands to Implement

| Subcommand | Description |
|------------|-------------|
| `omni` (no subcommand) | Launch TUI |
| `omni serve` | Start web server on localhost:9420 |
| `omni <task>` | Run single agent task headlessly |
| `omni swarm <task>` | Run swarm mode |
| `omni sentinel watch` | CI/CD sentinel daemon |
| `omni sentinel prod-watch` | Incident whisperer daemon |
| `omni skill install <name>` | Install skill from marketplace |
| `omni skill publish <path>` | Publish a skill |
| `omni review install` | Set up PR review webhook |
| `omni heal` | Run self-healing on test failures |
| `omni release <description>` | Prompt-to-binary pipeline |
| `omni archeologist --migrate <src> <tgt>` | Migration mode |
| `omni onboard` | Onboarding wizard |
| `omni guardian` | Dependency guardian daemon |
| `omni why <query>` | Decision historian query |
| `omni find <query>` | Semantic code search |
| `omni share` | Create collaboration session |
| `omni join <code>` | Join collaboration session |
| `omni config` | Open config in editor |
| `omni docs` | Regenerate documentation |
| `omni deploy` | Run deployment pipeline |

## Verification

After implementation, verify with:
```bash
cd omnicode && cargo check