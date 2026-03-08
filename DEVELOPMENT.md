# Development

## Prerequisites

- Rust 1.70+ (stable)

## Build

```
cargo build
cargo build --features binary   # includes the CLI
```

## Test

```
cargo test
```

The test suite includes:
- Unit tests for the parser state machine (`src/parser.rs`)
- Unit tests for the output rewriter (`src/runner.rs`)
- Integration tests that run the full suite against a small fake compiler (`tests/integration.rs`)

## Project layout

```
src/
  lib.rs       public API: Suite builder, run(), run_from_cwd()
  config.rs    ouro.toml parsing, upward config search
  patterns.rs  PatternSet trait + DefaultPatterns
  parser.rs    line scanner / state machine → TestCase
  runner.rs    spawn binary, capture output, compare, --update rewriter
  diff.rs      colored unified diff output
  main.rs      CLI entry point (binary feature)

tests/
  integration.rs       end-to-end integration tests
  fixtures/myc         minimal fake compiler used by integration tests
  golden/              golden test files for the integration suite
```

## Adding a new directive

1. Add a method to `PatternSet` in `src/patterns.rs`
2. Implement it in `DefaultPatterns`
3. Handle the new state/transition in `src/parser.rs`
4. Add a unit test in the `parser::tests` module

## Releasing

Releases are managed by [release-plz](https://release-plz.dev). No separate release branch is needed.

### How it works

1. When a PR is merged to `main`, release-plz opens a **release PR** that bumps the version in `Cargo.toml` and updates `CHANGELOG.md`.
2. When that release PR is merged, release-plz:
   - Creates a GitHub release with the generated changelog
   - Publishes the crate to [crates.io](https://crates.io)
3. The `release.yml` workflow triggers on the published GitHub release and builds cross-platform binaries (`linux-x86_64`, `macos-x86_64`, `macos-aarch64`, `windows-x86_64`), attaching them to the release.

### Required secrets

Add these in **Settings → Secrets and variables → Actions**:

| Secret | Where to get it |
|--------|----------------|
| `CARGO_REGISTRY_TOKEN` | [crates.io](https://crates.io/settings/tokens) → New token (scope: `publish-new`, `publish-update`) |

`GITHUB_TOKEN` is provided automatically by GitHub Actions.
