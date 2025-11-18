#!/usr/bin/env bash
# Installs the Rust toolchain (via rustup), mdBook, and cargo-make.
# Can be re-run safely; existing installations are reused.

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

ensure_rustup() {
    if command -v rustup >/dev/null 2>&1; then
        echo "rustup already installed."
        return
    fi

    echo "Installing rustup + stable toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
}

install_toolchain() {
    # shellcheck disable=SC1091
    source "$CARGO_HOME/env"
    rustup default stable
}

install_cargo_crate() {
    local crate="$1"
    if cargo install --list | grep -q "^${crate} v"; then
        echo "${crate} already installed."
        return
    fi
    echo "Installing ${crate}..."
    cargo install "$crate" --locked
}

main() {
    echo "Workspace root: ${WORKSPACE_ROOT}"
    ensure_rustup
    install_toolchain

    # shellcheck disable=SC1091
    source "$CARGO_HOME/env"

    install_cargo_crate mdbook
    install_cargo_crate cargo-make

    echo "Tooling installation complete."
}

main "$@"
