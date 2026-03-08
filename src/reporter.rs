use crate::diff::{print_diff, print_exit_mismatch};
use crate::runner::TestOutcome;
use colored::Colorize;
use std::path::Path;

pub trait Reporter: Sync {
    fn on_test_complete(&self, _path: &Path, _outcome: &TestOutcome) {}
    fn on_suite_complete(&self, _passed: usize, _failed: usize) {}
}

pub struct ConsoleReporter;

impl Reporter for ConsoleReporter {
    fn on_test_complete(&self, path: &Path, outcome: &TestOutcome) {
        match outcome {
            TestOutcome::Pass => {}
            TestOutcome::Updated => {
                eprintln!("updated {}", path.display());
            }
            TestOutcome::Fail {
                diffs,
                exit_mismatch,
                ..
            } => {
                eprintln!("\n{} {}", "FAIL".red().bold(), path.display());
                eprintln!();
                for d in diffs {
                    print_diff(d.stream, &d.expected, &d.actual);
                }
                if let Some((exp, act)) = exit_mismatch {
                    print_exit_mismatch(*exp, *act);
                }
            }
        }
    }

    fn on_suite_complete(&self, passed: usize, failed: usize) {
        if failed == 0 {
            eprintln!("test result: ok. {passed} passed; 0 failed");
        } else {
            eprintln!("test result: FAILED. {passed} passed; {failed} failed");
        }
    }
}
