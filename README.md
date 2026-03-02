# ouro

A golden test runner for language hackers. Embed test expectations directly in your source files as comments — no separate fixture files, no test harness boilerplate.

```
test tests/golden/simple.myc ... ok
test tests/golden/errors.myc ... ok
test tests/golden/multiline.myc ... ok

test result: ok. 3 passed; 0 failed
```

---

## How it works

ouro runs a binary (your compiler, interpreter, or language tool) against each test file and compares the output to expectations written in the file's own comments.

```javascript
// args: --optimize
// out: 42
// :out
let x = 42
console.log(x)
```

Directives live in comment lines starting with a configurable prefix (`// ` by default). Everything else is source code passed to your binary.

---

## Directives

| Directive | Form | Meaning |
|-----------|------|---------|
| `out: <text>` | inline | Entire stdout must equal `<text>` |
| `out:` / `:out` | block | Multi-line stdout expectation |
| `err: <text>` | inline | Entire stderr must equal `<text>` |
| `err:` / `:err` | block | Multi-line stderr expectation |
| `args: <flags>` | inline | Arguments passed to the binary (accumulates) |
| `args:` / `:args` | block | Multi-line args, one per line |
| `exit: <n>` | inline | Expected exit code (default: `0`) |

Omitting `out:` or `err:` means that stream is not checked.

### Inline shorthand

```python
# args: --run
# out: hello world
# exit: 0

print("hello world")
```

### Multi-line block

```c
// out:
// ; optimized output
// mov rax, 42
// ret
// :out
```

Block content can contain anything — including `}`, `//`, or other tokens from your language — without ambiguity.

---

## Setup

### 1. Add to `Cargo.toml`

```toml
[dev-dependencies]
ouro = "0.1"
```

### 2. Create `ouro.toml` in your project root

```toml
binary = "target/debug/myc"   # path to your binary
files  = "tests/**/*.myc"     # glob of test files
prefix = "// "                # comment prefix (default: "// ")
```

### 3. Write a test

```rust
// tests/golden.rs
#[test]
fn golden() {
    ouro::run_from_cwd().unwrap();
}
```

Or use the builder directly:

```rust
#[test]
fn golden() {
    ouro::Suite::new()
        .binary("target/debug/myc")
        .files("tests/**/*.myc")
        .prefix("// ")
        .run()
        .unwrap();
}
```

### 4. Run

```
cargo test golden
```

---

## CLI

Install the `ouro` binary with:

```
cargo install ouro --features binary
```

```
ouro [OPTIONS]

  --binary <PATH>    Binary to test
  --files <GLOB>     Test file glob     [default: tests/**/*]
  --prefix <STR>     Comment prefix     [default: "// "]
  --update           Overwrite expected output with actual
  --jobs <N>         Parallel workers   [default: num CPUs]
```

Exit 0 if all tests pass, 1 if any fail.

### Updating expectations

When your output intentionally changes, regenerate all expected values in one step:

```
ouro --update
```

This rewrites the directive lines in each test file with the actual output from your binary. Review the diff with `git diff`, then commit.

---

## Changing the comment prefix

For languages with a different comment syntax, set `prefix` in `ouro.toml` or via `--prefix`:

```toml
# Python / Ruby / shell
prefix = "# "
```

```toml
-- Lua / Haskell
prefix = "-- "
```

```toml
; Assembly / .ini
prefix = "; "
```

---

## Parallelism

Tests run in parallel by default using Rayon. Control the thread count:

```toml
# ouro.toml
jobs = 4
```

```
ouro --jobs 4
```

Disable parallelism entirely by depending on ouro without the `parallel` feature:

```toml
ouro = { version = "0.1", default-features = false }
```

---

## Cargo features

| Feature | Default | Description |
|---------|---------|-------------|
| `parallel` | yes | Parallel test execution via Rayon |
| `binary` | no | Build the `ouro` CLI binary (implies `parallel`) |

---

## Development

### Prerequisites

- Rust 1.70+ (stable)

### Build

```
cargo build
cargo build --features binary   # includes the CLI
```

### Test

```
cargo test
```

The test suite includes:
- Unit tests for the parser state machine (`src/parser.rs`)
- Unit tests for the output rewriter (`src/runner.rs`)
- Integration tests that run the full suite against a small fake compiler (`tests/integration.rs`)

### Project layout

```
src/
  lib.rs       public API: Suite builder, run(), run_from_cwd()
  config.rs    ouro.toml parsing, upward config search
  patterns.rs  PatternSet trait + DefaultPatterns
  parser.rs    line scanner / state machine → TestCase
  runner.rs    spawn binary, capture output, compare, --update rewriter
  diff.rs      colored unified diff output
  main.rs      CLI entry point (binary feature)
k
tests/
  integration.rs       end-to-end integration tests
  fixtures/myc         minimal fake compiler used by integration tests
  golden/              golden test files for the integration suite
```

### Adding a new directive

1. Add a method to `PatternSet` in `src/patterns.rs`
2. Implement it in `DefaultPatterns`
3. Handle the new state/transition in `src/parser.rs`
4. Add a unit test in the `parser::tests` module

---

## Prior art

[jfecher/golden-tests](https://github.com/jfecher/golden-tests)

## License

MIT
