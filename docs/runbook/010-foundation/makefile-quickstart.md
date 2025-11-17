# Makefile Quickstart - DFPS

This quickstart lists the most useful `cargo make` commands for day-to-day work.

## Common tasks

```bash
cargo make build         # cargo build --workspace
cargo make check         # cargo check --workspace
cargo make fmt           # cargo fmt --all
cargo make clippy        # cargo clippy -- -D warnings
cargo make test          # cargo test --workspace
cargo make ci            # fmt + clippy + test
cargo make clean
```

## Docs

```bash
cargo make docs          # build mdBook to docs/book/book
cargo make docs-serve    # live-reload server (mdbook serve)
```

## Web

```bash
cargo make api           # run dfps_api (backend)
cargo make web           # run dfps_web_frontend; /docs redirects to DFPS_DOCS_URL
```

## CLIs

```bash
cargo make map-bundles INPUT=path/to/bundles.ndjson
cargo make map-codes    INPUT=path/to/staging_codes.ndjson
```

### Notes

- The active env profile defaults to `dev`. Override with `DFPS_ENV=test cargo make test`.
- Install `cargo-make` once via `cargo install cargo-make`.
- Install `mdbook` via `cargo install mdbook` if you plan to run `cargo make docs` / `docs-serve`.
