//! Golden test runner for language authors.
//!
//! ouro runs a binary against a set of test files and compares its output to
//! expectations embedded in the files' own comments. See the
//! [README](https://github.com/NickTomlin/ouro) for the full directive reference
//! and CLI documentation.
//!
//! This crate lets you drive ouro from a `cargo test` test function as an
//! alternative to (or alongside) the standalone CLI.
//!
//! # Setup
//!
//! Add to `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! ouro = "0.1"
//! ```
//!
//! Create `ouro.toml` in your project root:
//!
//! ```toml
//! binary = "target/debug/myc"
//! files  = "tests/**/*.myc"
//! ```
//!
//! Write a test function:
//!
//! ```rust,no_run
//! #[test]
//! fn golden() {
//!     ouro::run_from_cwd().unwrap();
//! }
//! ```
//!
//! # Builder
//!
//! Use [`Suite`] to configure programmatically instead of (or to override) `ouro.toml`:
//!
//! ```rust,no_run
//! #[test]
//! fn golden() {
//!     ouro::Suite::new()
//!         .binary("target/debug/myc")
//!         .files("tests/**/*.myc")
//!         .run()
//!         .unwrap();
//! }
//! ```
//!
//! # Cargo features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `parallel` | yes | Parallel test execution via Rayon |
//! | `binary` | no | Build the `ouro` CLI binary (implies `parallel`) |

pub mod config;
pub mod diff;
pub mod parser;
pub mod patterns;
pub mod reporter;
pub mod runner;

use config::Config;
use parser::{parse_file, TestCase};
use patterns::DefaultPatterns;
use reporter::{ConsoleReporter, Reporter};
use runner::{run_test, update_test, TestOutcome};

use std::path::{Path, PathBuf};

/// Returned when one or more tests fail. Details have already been printed to stderr.
#[derive(Debug)]
pub struct TestsFailed;

impl std::fmt::Display for TestsFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "one or more tests failed")
    }
}

impl std::error::Error for TestsFailed {}

/// Load config from `ouro.toml` (searched upward from CWD) and run all tests.
pub fn run(config_path: impl AsRef<Path>) -> Result<(), TestsFailed> {
    let config = Config::from_file(config_path.as_ref()).map_err(|e| {
        eprintln!("ouro: failed to load config: {e}");
        TestsFailed
    })?;
    Suite::from_config(config).run()
}

/// Builder for running a suite of golden tests.
pub struct Suite {
    binary: Option<PathBuf>,
    files: String,
    prefix: String,
    jobs: Option<usize>,
    update: bool,
}

impl Suite {
    pub fn new() -> Self {
        Self {
            binary: None,
            files: "tests/**/*".to_string(),
            prefix: "// ".to_string(),
            jobs: None,
            update: false,
        }
    }

    fn from_config(config: Config) -> Self {
        Self {
            binary: Some(PathBuf::from(&config.binary)),
            files: config.files,
            prefix: config.prefix,
            jobs: config.jobs,
            update: false,
        }
    }

    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary = Some(path.into());
        self
    }

    pub fn files(mut self, glob: impl Into<String>) -> Self {
        self.files = glob.into();
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn jobs(mut self, n: usize) -> Self {
        self.jobs = Some(n);
        self
    }

    pub fn update(mut self, update: bool) -> Self {
        self.update = update;
        self
    }

    pub fn run(self) -> Result<(), TestsFailed> {
        let binary = self.binary.as_deref().ok_or_else(|| {
            eprintln!("ouro: no binary specified");
            TestsFailed
        })?;

        #[cfg(feature = "parallel")]
        if let Some(n) = self.jobs {
            rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global()
                .ok();
        }

        let patterns = DefaultPatterns::new(&self.prefix);
        let test_cases = collect_tests(&self.files, &patterns);

        if test_cases.is_empty() {
            eprintln!("ouro: no test files found matching '{}'", self.files);
            return Ok(());
        }

        let reporter = ConsoleReporter;
        let update = self.update;
        let results = run_all(&test_cases, binary, update, &reporter);

        let passed = results
            .iter()
            .filter(|r| matches!(r, TestOutcome::Pass | TestOutcome::Updated))
            .count();
        let failed = results.len() - passed;

        reporter.on_suite_complete(passed, failed);
        if failed == 0 {
            Ok(())
        } else {
            Err(TestsFailed)
        }
    }
}

impl Default for Suite {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_tests(glob_pattern: &str, patterns: &DefaultPatterns) -> Vec<TestCase> {
    let mut cases = Vec::new();
    let entries = glob::glob(glob_pattern).unwrap_or_else(|e| {
        eprintln!("ouro: invalid glob pattern: {e}");
        glob::glob("").unwrap()
    });

    for entry in entries.flatten() {
        if entry.is_file() {
            match parse_file(&entry, patterns) {
                Ok(tc) => cases.push(tc),
                Err(e) => eprintln!("ouro: parse error in {}: {e}", entry.display()),
            }
        }
    }
    cases
}

#[cfg(not(feature = "parallel"))]
fn run_all(
    cases: &[TestCase],
    binary: &Path,
    update: bool,
    reporter: &dyn Reporter,
) -> Vec<TestOutcome> {
    cases
        .iter()
        .map(|tc| run_one(tc, binary, update, reporter))
        .collect()
}

#[cfg(feature = "parallel")]
fn run_all(
    cases: &[TestCase],
    binary: &Path,
    update: bool,
    reporter: &dyn Reporter,
) -> Vec<TestOutcome> {
    use rayon::prelude::*;
    cases
        .par_iter()
        .map(|tc| run_one(tc, binary, update, reporter))
        .collect()
}

fn run_one(tc: &TestCase, binary: &Path, update: bool, reporter: &dyn Reporter) -> TestOutcome {
    let outcome = if update {
        match update_test(tc, binary) {
            Ok(()) => TestOutcome::Updated,
            Err(e) => {
                eprintln!("ouro: failed to update {}: {e}", tc.path.display());
                TestOutcome::Fail {
                    path: tc.path.clone(),
                    diffs: vec![],
                    exit_mismatch: None,
                }
            }
        }
    } else {
        match run_test(tc, binary) {
            Ok(outcome) => outcome,
            Err(e) => {
                eprintln!("ouro: failed to run {}: {e}", tc.path.display());
                TestOutcome::Fail {
                    path: tc.path.clone(),
                    diffs: vec![],
                    exit_mismatch: None,
                }
            }
        }
    };
    reporter.on_test_complete(&tc.path, &outcome);
    outcome
}

/// Convenience: load config from ouro.toml (searched upward from CWD).
pub fn run_from_cwd() -> Result<(), TestsFailed> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let config_path = config::find_config_file(&cwd).ok_or_else(|| {
        eprintln!("ouro: no ouro.toml found");
        TestsFailed
    })?;
    run(config_path)
}
