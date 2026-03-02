pub mod config;
pub mod diff;
pub mod parser;
pub mod patterns;
pub mod runner;

use config::Config;
use diff::{print_diff, print_exit_mismatch};
use parser::{parse_file, TestCase};
use patterns::DefaultPatterns;
use runner::{run_test, update_test, TestOutcome};

use std::path::{Path, PathBuf};

/// Load config from `ouro.toml` (searched upward from CWD) and run all tests.
pub fn run(config_path: impl AsRef<Path>) -> Result<(), ()> {
    let config = Config::from_file(config_path.as_ref()).map_err(|e| {
        eprintln!("ouro: failed to load config: {e}");
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

    pub fn run(self) -> Result<(), ()> {
        let binary = self.binary.as_deref().ok_or_else(|| {
            eprintln!("ouro: no binary specified");
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

        let update = self.update;
        let results = run_all(&test_cases, binary, update);

        let passed = results.iter().filter(|r| matches!(r, TestOutcome::Pass)).count();
        let failed = results.len() - passed;

        if failed == 0 {
            eprintln!("\ntest result: ok. {passed} passed; 0 failed");
            Ok(())
        } else {
            eprintln!("\ntest result: FAILED. {passed} passed; {failed} failed");
            Err(())
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
fn run_all(cases: &[TestCase], binary: &Path, update: bool) -> Vec<TestOutcome> {
    cases.iter().map(|tc| run_one(tc, binary, update)).collect()
}

#[cfg(feature = "parallel")]
fn run_all(cases: &[TestCase], binary: &Path, update: bool) -> Vec<TestOutcome> {
    use rayon::prelude::*;
    cases.par_iter().map(|tc| run_one(tc, binary, update)).collect()
}

fn run_one(tc: &TestCase, binary: &Path, update: bool) -> TestOutcome {
    if update {
        match update_test(tc, binary) {
            Ok(()) => {
                eprintln!("updated {}", tc.path.display());
                TestOutcome::Pass
            }
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
            Ok(outcome) => {
                match &outcome {
                    TestOutcome::Pass => {
                        eprintln!("test {} ... ok", tc.path.display());
                    }
                    TestOutcome::Fail { path, diffs, exit_mismatch } => {
                        eprintln!("test {} ... FAIL", path.display());
                        for d in diffs {
                            print_diff(d.stream, &d.expected, &d.actual);
                        }
                        if let Some((exp, act)) = exit_mismatch {
                            print_exit_mismatch(*exp, *act);
                        }
                    }
                }
                outcome
            }
            Err(e) => {
                eprintln!("ouro: failed to run {}: {e}", tc.path.display());
                TestOutcome::Fail {
                    path: tc.path.clone(),
                    diffs: vec![],
                    exit_mismatch: None,
                }
            }
        }
    }
}

/// Convenience: load config from ouro.toml (searched upward from CWD).
pub fn run_from_cwd() -> Result<(), ()> {
    let cwd = std::env::current_dir().unwrap_or_default();
    let config_path = config::find_config_file(&cwd).ok_or_else(|| {
        eprintln!("ouro: no ouro.toml found");
    })?;
    run(config_path)
}
