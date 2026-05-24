# CLAUDE.md — Small Project

## Project: `small`

An agent-inspired CLI tool that lets developers wrap up their project so users can:

```
git clone <repo> && small install && small run
```

No Docker, no Nix, no environment knowledge required. Small orchestrates existing ecosystems (pip/npm/apt/brew) — it does NOT replace them.

## Tech Stack

- **Language**: Rust (stable, edition 2024)
- **Binary target**: single static binary, ~5-8MB after LTO+strip
- **Platforms**: macOS (arm64 + x64) + Linux (x64)
- **No Windows support** in MVP

## Critical Design Rules

These are NON-NEGOTIABLE. Enforce them in every change:

1. **No traits, no abstractions.** Hardcode everything. Only extract common patterns after 50+ real projects pass `small install`.
2. **No lockfile.** Small calls pip/npm — they handle version pinning.
3. **No checkpoint/resume.** If install fails, clean up and retry from scratch.
4. **No LLM/AI diagnostics.** No real failure data exists yet.
5. **No platform_overrides in schema.** MVP doesn't need it.
6. **No comments** unless the WHY is genuinely non-obvious. No docstrings, no module headers.

## Architecture (Phase 1 — MVP)

```
src/
├── main.rs       # CLI entry point (clap), subcommand dispatch
├── yaml.rs       # small.yaml parsing (serde_yaml)
├── init.rs       # small init: scan + interactive (dialoguer)
├── install.rs    # small install: main orchestration logic
├── runtime.rs    # Runtime detection + download (python-build-standalone, Node tarball)
├── syspkg.rs     # System package install (apt/brew)
├── plan.rs       # Execution plan generation + display
├── executor.rs   # Step-by-step plan execution
├── check.rs      # small check: dry-run only
├── clean.rs      # small clean: remove venv/node_modules
├── run.rs        # small run: launch via entrypoint
└── display.rs    # Terminal output (colors, progress with indicatif)

data/
├── versions.toml  # Runtime download URLs + SHA256 (compiled into binary)
└── packages.toml  # System package name mappings (compiled into binary)
```

## Core Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI derive macros |
| `serde` + `serde_yaml` | YAML parsing |
| `toml` | Embedded data files |
| `dialoguer` | Interactive `small init` prompts |
| `reqwest` + `tokio` | HTTP downloads (async) |
| `flate2` + `tar` | Runtime archive extraction |
| `sha2` | SHA256 checksum verification |
| `which` | Detect system tools (python, node, brew, apt) |
| `duct` | Shell command execution |
| `indicatif` | Progress bars |
| `console` | Terminal colors/styles |

## small.yaml Schema (v0.1)

```yaml
project:
  name: "my-project"       # required
  language: python         # required: python | node

runtimes:
  python: ">=3.9,<3.13"    # required for python
  # node: ">=18"           # required for node

system:                     # optional
  - cmake
  - pkg-config

dependencies:               # optional (or use requirements_file/package_file)
  python:
    - "numpy>=1.24"

requirements_file: "requirements.txt"  # optional
env:                                   # optional
  MY_KEY: "please fill in"
pre_install:                           # optional
  - "./scripts/setup.sh"
post_install:                          # optional
  - "python -m spacy download en_core_web_sm"
test: "python -m pytest tests/ -v"    # optional
entrypoint: "python main.py"          # required
```

## CLI Commands

| Command | Behavior |
|---------|----------|
| `small init` | Scan project files + interactive → generate small.yaml |
| `small install` | Detect → download runtimes → install system pkgs → venv/npm → dependencies → test |
| `small check` | Dry-run: check environment, report what would be installed |
| `small clean` | Remove .small_venv / node_modules (preserves runtime cache) |
| `small run` | Execute entrypoint from small.yaml |
| `small version` | Print version |

## Execution Flow (small install)

```
parse_yaml → check_os → check_python/node → check_system_pkgs
→ build_plan → print_plan → [user confirms Y/N]
→ execute: download_runtime → install_system_pkgs → create_venv
→ pip/npm_install → run_test → done
```

On failure: print original error + `rm -rf .small_venv node_modules`. System packages are NOT rolled back.

## Runtime Cache

```
~/.small/runtimes/
├── python/3.11.9/...
├── node/20.12.0/...
└── versions.toml
```

- Check system PATH first → if version constraint satisfied, use system
- Otherwise download to `~/.small/runtimes/`, add to PATH during install
- `small clean` does NOT delete runtime cache (shared across projects)

## Benchmark-Driven Development

The FIRST priority: `small install` must pass on 20+ real open-source projects before MVP ships. Each benchmark: `small install` → `small run` → `small test` → `small clean`.

## CI Requirements

- Build/test on macOS (arm64) + Linux (x64)
- `cargo fmt --check` + `cargo clippy -- -D warnings`
- On tag push: build release binaries, create GitHub Release with curl install script

## Notes for Claude

- The developer is a Rust beginner. Code should be correct and idiomatic but not over-explained.
- Prefer `edit` over `write` for existing files.
- Run `cargo check` after each meaningful code change to catch errors early.
- `cargo fmt` and `cargo clippy --fix` before committing.
- Prefer `duct` over `std::process::Command` for shell execution.
- Use `anyhow` for error handling (not thiserror) — simpler, good enough for CLI apps.
- Async via `tokio` only where needed (HTTP downloads). Keep the rest synchronous.
