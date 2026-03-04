#[cfg(feature = "binary")]
fn main() {
    use clap::{Parser, Subcommand};
    use ouro::{config, Suite};
    use std::path::PathBuf;

    #[derive(Parser)]
    #[command(
        name = "ouro",
        about = "Golden test runner for language hackers",
        long_about = "Golden test runner for language hackers.\n\
            \n\
            ouro runs a binary against source files and compares stdout, stderr, and exit\n\
            code to expectations embedded in the file's own comment lines. No separate\n\
            fixture files — test expectations live next to the source they describe.\n\
            \n\
            DIRECTIVE SYNTAX\n\
            \n\
            Each test file is a plain source file with comment lines containing directives.\n\
            The default comment prefix is \"// \" (configurable with --prefix).\n\
            \n\
              // out: hello world        stdout must equal \"hello world\"\n\
              // err: oops               stderr must equal \"oops\"\n\
              // exit: 1                 expected exit code (default 0)\n\
              // args: --flag value      flags appended to the binary invocation\n\
            \n\
            Multi-line blocks use an open/close pair:\n\
            \n\
              // out:\n\
              // line one\n\
              // line two\n\
              // :out\n\
            \n\
            Streams not mentioned in directives are not checked.\n\
            \n\
            INVOCATION\n\
            \n\
            For each test file ouro calls:  <binary> [args...] <file>\n\
            \n\
            CONFIG FILE\n\
            \n\
            ouro.toml is searched upward from the current directory:\n\
            \n\
              binary = \"target/debug/myc\"\n\
              files  = \"tests/**/*.myc\"\n\
              prefix = \"// \"\n\
              jobs   = 4\n\
            \n\
            CLI flags always override ouro.toml. Run 'ouro llm-context' for a full spec."
    )]
    struct Cli {
        #[command(subcommand)]
        command: Option<Commands>,

        /// Path to the binary under test (e.g. target/debug/myc). Overrides ouro.toml.
        #[arg(long)]
        binary: Option<PathBuf>,

        /// Glob of test files (e.g. "tests/**/*.myc"). Overrides ouro.toml.
        #[arg(long)]
        files: Option<String>,

        /// Comment prefix used to identify directive lines (e.g. "// ", "# ", "-- "). Overrides ouro.toml.
        #[arg(long)]
        prefix: Option<String>,

        /// Rewrite directive lines in each file with the actual output instead of failing.
        /// Use after intentional output changes; review results with `git diff`.
        #[arg(long, default_value_t = false)]
        update: bool,

        /// Number of parallel workers (default: number of logical CPUs). Overrides ouro.toml.
        #[arg(long)]
        jobs: Option<usize>,

        /// Path to ouro.toml config file. Defaults to searching upward from the current directory.
        #[arg(long)]
        config: Option<PathBuf>,
    }

    #[derive(Subcommand)]
    enum Commands {
        /// Print a self-contained spec of ouro's directive syntax and CLI, suitable for an LLM context window
        LlmContext,
    }

    let cli = Cli::parse();

    if let Some(Commands::LlmContext) = cli.command {
        print!("{}", llm_context());
        std::process::exit(0);
    }

    // Load base config from ouro.toml if present
    let base = {
        let config_path = cli.config.or_else(|| {
            let cwd = std::env::current_dir().unwrap_or_default();
            config::find_config_file(&cwd)
        });
        config_path.and_then(|p| config::Config::from_file(&p).ok())
    };

    // Build suite, overriding config with CLI flags
    let mut suite = Suite::new();

    if let Some(ref cfg) = base {
        suite = suite
            .binary(cfg.binary.clone())
            .files(cfg.files.clone())
            .prefix(cfg.prefix.clone());
        if let Some(j) = cfg.jobs {
            suite = suite.jobs(j);
        }
    }

    if let Some(b) = cli.binary {
        suite = suite.binary(b);
    }
    if let Some(f) = cli.files {
        suite = suite.files(f);
    }
    if let Some(p) = cli.prefix {
        suite = suite.prefix(p);
    }
    if let Some(j) = cli.jobs {
        suite = suite.jobs(j);
    }
    suite = suite.update(cli.update);

    match suite.run() {
        Ok(()) => std::process::exit(0),
        Err(_) => std::process::exit(1),
    }
}

fn llm_context() -> &'static str {
    r##"# ouro — golden test runner for language hackers

## What it does

ouro runs a binary (compiler, interpreter, or other language tool) against source
files and compares stdout, stderr, and exit code to expectations embedded in the
file's own comment lines. No separate fixture files. Test expectations live next
to the source code they describe.

## Binary invocation contract

For each test file, ouro calls:

    <binary> [args...] <test-file-path>

The test file path is ALWAYS the last argument. args come from `args:` directives
in the file (see below). If no args directives are present, ouro calls:

    <binary> <test-file-path>

## Directive syntax

Directives are comment lines whose text (after stripping the prefix) starts with
a keyword. The default prefix is "// ". Lines not starting with the prefix are
source code and are ignored by ouro.

### Keywords

    // out: <text>       Entire stdout must equal <text> (inline form)
    // out:              Open multi-line stdout block
    // :out              Close multi-line stdout block

    // err: <text>       Entire stderr must equal <text> (inline form)
    // err:              Open multi-line stderr block
    // :err              Close multi-line stderr block

    // args: <flags>     Append shell-split flags to args (accumulates; can repeat)
    // args:             Open multi-line args block (one arg per line)
    // :args             Close multi-line args block

    // exit: <n>         Expected exit code. Default: 0.

Omitting out: / err: means that stream is not checked at all.

### Inline example (single-line expectation)

    // args: --run
    // out: hello world
    // exit: 0

    print("hello world")

### Block example (multi-line expectation)

    // out:
    // ; optimized output
    // mov rax, 42
    // ret
    // }
    // :out

    ... source code ...

Block content can contain any text including }, //, and other language tokens.

## Comparison semantics

- Trailing newlines are trimmed from both expected and actual before comparison.
- Comparison is exact string match (no regex, no whitespace normalization).
- Streams not mentioned in directives are not checked.

## ouro.toml (config file, searched upward from CWD)

    binary = "target/debug/myc"   # required: path to the binary under test
    files  = "tests/**/*.myc"     # glob of test files (default: tests/**/*)
    prefix = "// "                # comment prefix (default: "// ")
    jobs   = 4                    # rayon thread count (default: num CPUs)

## CLI flags (all override ouro.toml)

    ouro [--binary <PATH>] [--files <GLOB>] [--prefix <STR>]
         [--update] [--jobs <N>] [--config <PATH>]
    ouro llm-context

    --update    Rewrite directive lines in each file with actual output.
                Use after intentional output changes; review with git diff.

## Exit codes

    0   all tests passed
    1   one or more tests failed

## Library API (Rust)

    // Minimal — reads ouro.toml from CWD upward:
    ouro::run_from_cwd().unwrap();

    // Builder — explicit config:
    ouro::Suite::new()
        .binary("target/debug/myc")
        .files("tests/**/*.myc")
        .prefix("// ")
        .run()                    // returns Ok(()) or Err(())
        .unwrap();                // Err means tests failed; details already on stderr

    // From a config file path:
    ouro::run("ouro.toml").unwrap();

run() / run_from_cwd() return Err(()) when one or more tests fail. The diff
output has already been printed to stderr before the return. There is no
structured error value to inspect; check stderr for details.

## Cargo features

    parallel   (default) parallel execution via rayon
    binary     build the ouro CLI (implies parallel)

## Comment prefix for other languages

    prefix = "# "    # Python, Ruby, shell
    prefix = "-- "   # Lua, Haskell, SQL
    prefix = "; "    # Assembly, Lisp, .ini
"##
}

#[cfg(not(feature = "binary"))]
fn main() {
    eprintln!("ouro: binary feature not enabled");
    std::process::exit(1);
}
