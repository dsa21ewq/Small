# Small — Progress & Benchmark Status

Last updated: 2026-05-26

## Completed Features

### CLI Commands

| Command | Status |
|---------|--------|
| `small init` | ✅ Interactive & `--yes` non-interactive mode |
| `small install` | ✅ Full orchestration: runtime → venv → deps → post_install |
| `small check` | ✅ Dry-run |
| `small clean` | ✅ Remove venv/node_modules, preserve runtime cache |
| `small run` | ✅ Execute entrypoint with venv/node_modules in PATH |
| `small test` | ✅ Run test command from small.yaml |
| `small version` | ✅ |

### Runtime Support

| Runtime | Detection | Download |
|---------|-----------|----------|
| Python (system) | `>=3.9` constraint matching | — |
| Python (download) | python-build-standalone | 3.11.15 (20260510 release) |
| Node (system) | `>=18` constraint matching | — |
| Node (download) | nodejs.org tarball | 20.12.0 |

### Package Managers

| System | Status |
|--------|--------|
| Homebrew (macOS) | ✅ Install, check |
| Apt (Linux) | ✅ Install via sudo, check via dpkg |

### Dependency Installation

| Method | Python | Node |
|--------|--------|------|
| `requirements.txt` | ✅ | — |
| Inline deps | ✅ | — |
| `npm install` | — | ✅ |
| Pre/post install hooks | ✅ | ✅ |

## Architecture (14 source modules)

```
src/
├── main.rs       # CLI entry point (clap)
├── yaml.rs       # small.yaml parsing (serde_yaml)
├── init.rs       # small init: scan + interactive
├── install.rs    # small install: orchestration
├── runtime.rs    # Runtime detection + download
├── syspkg.rs     # System package install (apt/brew)
├── plan.rs       # Execution plan generation
├── executor.rs   # Step-by-step execution
├── check.rs      # small check: dry-run
├── clean.rs      # small clean: artifact removal
├── run.rs        # small run: entrypoint launch
├── test.rs       # small test: test command execution
└── display.rs    # Terminal output (colors, progress)
```

## CI Pipelines

| Workflow | Purpose | Status |
|----------|---------|--------|
| `ci.yml` | Lint, format, clippy, build/test on macOS+Linux, release on tag | ✅ |
| `test-init.yml` | `small init` on 4 real projects (requests, flask, glances, express) | ✅ |
| `test-install.yml` | Full pipeline: `install → run → clean` on 23 projects | 🔶 18/23 passing |
| `test-bare-metal.yml` | Runtime download without system Python/Node | 🔶 1/2 passing |

## Benchmark Results (23 projects)

### Passing (18)

#### Python
- **glances** — requirements.txt → pip install . → glances CLI
- **cookiecutter** — pip install . → cookiecutter CLI
- **httpie** — pip install . → http CLI
- **click** — pip install . → import click
- **jinja** — pip install . → import jinja2
- **markupsafe** — pip install . (C extension)
- **requests** — pip install .
- **rich** — pip install . → import rich
- **pytest** — pip install . → pytest CLI

#### Node
- **express** — npm install
- **morgan** — npm install
- **cors** — npm install
- **body-parser** — npm install
- **commander** — npm install
- **chalk** — npm install
- **lodash** — npm install
- **debug** — npm install
- **axios** — npm install

### Failing (5)

| Project | Failure | Cause |
|---------|---------|-------|
| aiohttp | `small install -y` | Cython build fails with `pip install .` |
| flask | `small run` | `pip show flask` not found after install |
| werkzeug | `small run` | same |
| itsdangerous | `small run` | same |
| python-dotenv | `small run` | same |

## Key Bug Fixes

1. **Tarball extraction**: Strip top-level directory, create parent dirs before symlinks
2. **Node Linux URL**: .tar.xz → .tar.gz (avoid xz2 dependency)
3. **node_modules/.bin in PATH**: Added for `small run` and `small test`
4. **Downloaded Node runtime PATH**: Prepend for `npm install` step
5. **versions.toml**: Updated Python URLs from stale 20240224 → 20260510 release

## Known Gaps

- [ ] SHA256 checksums: all `000...000` placeholders (downloads work but unverified)
- [ ] No `pyproject.toml` detection in `small init` (only requirements.txt)
- [ ] No `pip install .` native step (uses post_install workaround)
- [ ] Zero Rust unit tests
- [ ] No macOS bare-metal CI test
- [ ] No Windows support (out of MVP scope)
