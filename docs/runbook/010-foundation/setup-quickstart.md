# Initial Setup Quickstart

This guide walks you through setting up the refractive_swan development environment from scratch.

## Prerequisites

- **Git**: Clone the repository
- **Bash**: Available on Linux/macOS by default; on Windows, use Git Bash or WSL
- **Internet connection**: For downloading Rust toolchain and dependencies

## Quick Start (New Contributors)

If you're setting up the project for the first time, run these commands in order:

```bash
# 1. Install Rust toolchain and development tools
cargo make setup-tooling

# 2. Create environment configuration files
cargo make setup-env

# 3. Validate your setup
cargo make validate-env

# 4. Build the project
cargo make build

# 5. Run tests to verify everything works
cargo make test
```

---

## Optional: Production Shell Bootstrap (zsh + Oh My Zsh)

Need a consistent terminal with Git/Rust/Docker helpers and a Powerlevel10k prompt? Run the shell bootstrap script before diving into the rest of the guide:

```bash
bash data/ops/scripts/setup_shell.sh
```

> **Devcontainer note:** the custom `.devcontainer/Dockerfile` pre-installs `zsh` + Rust tooling and sets it as the VS Code default shell. BuildKit cache mounts and persistent volumes for Cargo registry/git/target keep rebuilds fast—after the first build, most layers are reused. Rebuild only when you modify the Dockerfile or `install_rust_tooling.sh`.

What the script does:
- Installs/upgrades **Oh My Zsh**, **Powerlevel10k**, `zsh-autosuggestions`, and `zsh-syntax-highlighting`
- Drops a reproducible `.zshrc` tuned for Git, Rust (`cargo`/`rustup`), Docker, devcontainers, and Codex CLI utilities
- Installs Meslo Nerd Fonts locally (Linux/macOS) and refreshes `fc-cache` when needed
- Creates a matching `.p10k.zsh` so the prompt renders consistently without running the interactive wizard

After the script completes:
- Run `chsh -s "$(command -v zsh)"` if you want zsh to be your default shell
- Set your terminal profile to use the `MesloLGS NF` font family
- Restart the terminal to pick up the new config

Re-running the script is safe—existing dotfiles are backed up as `*.bak.<timestamp>` before being replaced.

---

## Step-by-Step Guide

### 1. Install Rust Toolchain & Development Tools

The `setup-tooling` task installs:
- **rustup** (Rust toolchain manager)
- **cargo-make** (Task runner)
- **mdbook** (Documentation builder)
- **cargo-watch** (File watcher for development)
- **cargo-insta** (Snapshot testing tool)

```bash
cargo make setup-tooling
```

**What it does:**
- Runs `data/ops/scripts/install_rust_tooling.sh`
- Installs Rust stable toolchain if not present
- Installs all required cargo crates
- Safe to re-run (skips already installed tools)

**Manual alternative:**
```bash
bash data/ops/scripts/install_rust_tooling.sh
```

---

### 2. Configure Environment Files

refractive_swan uses namespace-based `.env` files for configuration. The `setup-env` wizard creates these from example templates:

```bash
cargo make setup-env
```

**What it creates:**
- `data/environment/.env.app.web.api.dev`
- `data/environment/.env.app.web.frontend.dev`
- `data/environment/.env.app.cli.dev`

**Customization:**
Edit the generated files to configure:
- Database paths
- API endpoints
- Feature flags
- Logging levels

See [env-quickstart.md](./env-quickstart.md) for detailed configuration options.

---

### 3. Validate Your Setup

Verify all required environment files are present:

```bash
cargo make validate-env
```

**Expected output:**
```
Validating refractive_swan environment files...
  ✓ Found: data/environment/.env.app.web.api.dev
  ✓ Found: data/environment/.env.app.web.frontend.dev
  ✓ Found: data/environment/.env.app.cli.dev
✅ All required env files present
```

---

### 4. Build the Project

Compile all crates in the workspace:

```bash
cargo make build
```

This runs `cargo build --all` and ensures all dependencies are fetched and compiled.

---

### 5. Run Tests

Verify the build is working correctly:

```bash
cargo make test
```

> The `refractive_swan_test_suite` now auto-detects eval fixtures even when `refractive_swan_EVAL_DATA_ROOT` is unset or accidentally points at the parent `.../data` directory. Override the root only if you store custom NDJSON/manifest files elsewhere, and ensure that directory exists.

**Snapshot tests:**
If snapshot tests fail (expected after UI changes), review and update them:

```bash
cargo insta test --review
```

---

## Next Steps

Once your environment is set up, explore these workflows:

### Development Workflows
- **Run the web frontend + API**: `cargo make mvp-up`
- **Watch mode (auto-rebuild)**: `cargo make mvp-dev-loop`
- **Run specific services**: `cargo make web` or `cargo make api`

### Documentation
- **Build docs**: `cargo make docs`
- **Serve docs locally**: `cargo make docs-serve`

### Data Pipelines
- **Quick mapping evaluation**: `cargo make eval-quick`
- **Validate FHIR bundles**: `cargo make fhir-validate`
- **Load data to warehouse**: `cargo make warehouse-load`

See [makefile-quickstart.md](./makefile-quickstart.md) for a complete list of available tasks.

---

## Troubleshooting

### "cargo-make not found"
Run `setup-tooling` again or manually install:
```bash
cargo install cargo-make --locked
```

### "Missing .env files"
Run `setup-env` to create them from examples:
```bash
cargo make setup-env
```

### "Rust toolchain not installed"
Install rustup manually:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows: "bash not found"
Install Git for Windows (includes Git Bash) or use WSL.

---

## Environment Profiles

By default, tasks use the `dev` profile. To use a different profile:

```bash
export refractive_swan_ENV=prod
cargo make validate-env
```

Supported profiles:
- `dev` (default) - Local development
- `test` - CI/testing
- `prod` - Production configuration

---

## Advanced: Custom Namespaces

To create a new service with its own environment configuration:

```bash
NAMESPACE=app.new.service cargo make env-template
```

This generates `.env.app.new.service.example` which you can copy to `.env.app.new.service.dev`.
