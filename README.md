# 🥇 ouro 🥇

A golden test runner for language authors. Embed test expectations directly in your source files as comments — no separate fixture files, no test harness boilerplate.

```
test tests/golden/simple.myc ... ok
test tests/golden/errors.myc ... ok
test tests/golden/multiline.myc ... ok

test result: ok. 3 passed; 0 failed
```

---

## Quick start

### 1. Install

```sh
curl -sSfL https://raw.githubusercontent.com/NickTomlin/ouro/main/install.sh | bash
```

### 2. Create `ouro.toml`

```toml
binary = "./myc"            # your compiler or interpreter
files  = "tests/**/*.myc"  # glob of test files
```

### 3. Annotate your test files

ouro runs `<binary> <test-file>` for each file and compares its output to directives written in the file's own comments:

```javascript
// args: --optimize
// out: 42

let x = 42
console.log(x)
```

### 4. Run

```
ouro
```

That's it. Exit 0 if all tests pass, 1 if any fail.

---

## Directives

| Directive | Form | Meaning |
|-----------|------|---------|
| `out: <text>` | inline | Entire stdout must equal `<text>` |
| `out:` / `:out` | block | Multi-line stdout expectation |
| `err: <text>` | inline | Entire stderr must equal `<text>` |
| `err:` / `:err` | block | Multi-line stderr expectation |
| `args: <flags>` | inline | Append shell-split flags to args (can repeat) |
| `args:` / `:args` | block | Multi-line args, one arg per line |
| `exit: <n>` | inline | Expected exit code (default: `0`) |

Omitting `out:` or `err:` means that stream is not checked. Trailing newlines are trimmed before comparing; everything else is an exact match.

**Multi-line block:**

```c
// out:
// ; optimized output
// mov rax, 42
// ret
// :out
```

Block content can contain anything — including `}`, `//`, or other tokens from your language — without ambiguity.

---

## Configuration

### `ouro.toml`

```toml
binary = "./myc"           # required: path to your binary
files  = "tests/**/*.myc" # required: glob of test files
prefix = "// "             # comment prefix (default: "// ")
jobs   = 4                 # parallel workers (default: num CPUs)
```

### Comment prefix

Match the comment syntax of your language:

```toml
prefix = "# "    # Python / Ruby / shell
prefix = "-- "   # Lua / Haskell
prefix = "; "    # Assembly / .ini
```

### CLI flags

All config options can be passed as flags and override `ouro.toml`:

```
ouro [OPTIONS]

  --binary <PATH>    Binary to test
  --files <GLOB>     Test file glob
  --prefix <STR>     Comment prefix
  --update           Overwrite expected output with actual
  --jobs <N>         Parallel workers
  --config <PATH>    Path to ouro.toml  [default: search upward from CWD]
```

### Updating expectations

When your output intentionally changes, regenerate all expected values in one step:

```
ouro --update
```

This rewrites the directive lines in each test file with the actual output from your binary. Review the diff with `git diff`, then commit.

---

## CI

**GitHub Actions:**

```yaml
- name: Install ouro
  run: curl -sSfL https://raw.githubusercontent.com/NickTomlin/ouro/main/install.sh | bash

- name: Run golden tests
  run: ouro
```

On Windows runners, or to pin a specific version, download a release asset directly. Each [GitHub release](https://github.com/NickTomlin/ouro/releases) includes: `ouro-linux-x86_64`, `ouro-macos-x86_64`, `ouro-macos-aarch64`, `ouro-windows-x86_64.exe`.

---

## Rust crate

See [docs.rs/ouro](https://docs.rs/ouro) for the Rust API.

---

## Contributing

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions, project layout, and release process.

---

## Prior art

[jfecher/golden-tests](https://github.com/jfecher/golden-tests)

## License

MIT
