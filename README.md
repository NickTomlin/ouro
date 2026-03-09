# 🥇 ouro 🥇

A golden test runner for language authors. Embed test expectations directly in your source files as comments — no framework required.

```sh
test tests/golden/errors.example ... ok
test tests/golden/multiline.example ... ok
FAIL tests/golden/simple.example

  stdout:
    - wrong expected output
    + hello world

test result: FAILED. 2 passed; 1 failed
```

# Quick start

### Install

```sh
curl -sSfL https://raw.githubusercontent.com/NickTomlin/ouro/main/install.sh | bash
```

(See [github actions](#ci) integration for CI/CD).

### Create and annotate your test files

```
// out: hello world

print "hello world"
```

Or check stderr and exit code together:

```
// err: warning: unused variable 'x'
// exit: 1

error "warning: unused variable 'x'"
```

### 3. Run

```
ouro --binary ./example --files "tests/**/*.example"
```

You can use the [`ouro.toml` config file](#configuration) to avoid repeating flags.


# Directives

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

# Configuration

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

### `ouro.toml`

```toml
binary = "./example"           # required: path to your binary
files  = "tests/**/*.example" # required: glob of test files
prefix = "// "                 # comment prefix (default: "// ")
jobs   = 4                     # parallel workers (default: num CPUs)
```

### Comment prefix

Match the comment syntax of your language:

```toml
prefix = "# "    # Python / Ruby / shell
prefix = "-- "   # Lua / Haskell
prefix = "; "    # Assembly / .ini
```

### Updating expectations

When your output intentionally changes, regenerate all expected values in one step:

```
ouro --update
```

This rewrites the directive lines in each test file with the actual output from your binary. Review the diff with `git diff`, then commit.


# Other usage

## CI

**GitHub Actions:**

```yaml
- name: Install ouro
  run: curl -sSfL https://raw.githubusercontent.com/NickTomlin/ouro/main/install.sh | bash

- name: Run golden tests
  run: ouro
```

On Windows runners, or to pin a specific version, download a release asset directly. Each [GitHub release](https://github.com/NickTomlin/ouro/releases) includes: `ouro-linux-x86_64`, `ouro-macos-x86_64`, `ouro-macos-aarch64`, `ouro-windows-x86_64.exe`.


## Rust crate

See [docs.rs/ouro](https://docs.rs/ouro) for the Rust API.

---

# Contributing

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions, project layout, and release process.

---

# Prior art

[jfecher/golden-tests](https://github.com/jfecher/golden-tests)

# License

MIT
