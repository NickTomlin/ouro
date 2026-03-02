use std::path::{Path, PathBuf};
use std::process::Command;
use crate::parser::TestCase;

#[derive(Debug)]
pub struct Diff {
    pub stream: &'static str,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug)]
pub enum TestOutcome {
    Pass,
    Fail {
        path: PathBuf,
        diffs: Vec<Diff>,
        exit_mismatch: Option<(i32, i32)>, // (expected, actual)
    },
}

pub fn run_test(tc: &TestCase, binary: &Path) -> Result<TestOutcome, std::io::Error> {
    let output = Command::new(binary)
        .args(&tc.args)
        .arg(&tc.path)
        .output()?;

    let mut diffs = Vec::new();

    if let Some(ref expected) = tc.expected_stdout {
        let actual = String::from_utf8_lossy(&output.stdout).into_owned();
        // Trim trailing newline from actual for comparison
        let actual_trimmed = actual.trim_end_matches('\n').to_string();
        let expected_trimmed = expected.trim_end_matches('\n').to_string();
        if actual_trimmed != expected_trimmed {
            diffs.push(Diff {
                stream: "stdout",
                expected: expected_trimmed,
                actual: actual_trimmed,
            });
        }
    }

    if let Some(ref expected) = tc.expected_stderr {
        let actual = String::from_utf8_lossy(&output.stderr).into_owned();
        let actual_trimmed = actual.trim_end_matches('\n').to_string();
        let expected_trimmed = expected.trim_end_matches('\n').to_string();
        if actual_trimmed != expected_trimmed {
            diffs.push(Diff {
                stream: "stderr",
                expected: expected_trimmed,
                actual: actual_trimmed,
            });
        }
    }

    let actual_exit = output.status.code().unwrap_or(-1);
    let exit_mismatch = if actual_exit != tc.expected_exit {
        Some((tc.expected_exit, actual_exit))
    } else {
        None
    };

    if diffs.is_empty() && exit_mismatch.is_none() {
        Ok(TestOutcome::Pass)
    } else {
        Ok(TestOutcome::Fail {
            path: tc.path.clone(),
            diffs,
            exit_mismatch,
        })
    }
}

/// Update the test file in-place: rewrite expected stdout/stderr/exit in the file.
pub fn update_test(tc: &TestCase, binary: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(binary)
        .args(&tc.args)
        .arg(&tc.path)
        .output()?;

    let actual_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let actual_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let actual_exit = output.status.code().unwrap_or(-1);

    // Re-write the file, replacing expected directives with actual output
    rewrite_expected(
        &tc.path,
        tc.expected_stdout.is_some(),
        tc.expected_stderr.is_some(),
        actual_stdout.trim_end_matches('\n'),
        actual_stderr.trim_end_matches('\n'),
        actual_exit,
        tc.expected_exit,
    )?;
    Ok(())
}

fn rewrite_expected(
    path: &Path,
    had_stdout: bool,
    had_stderr: bool,
    new_stdout: &str,
    new_stderr: &str,
    new_exit: i32,
    old_exit: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let mut result = rewrite_directives(&content, had_stdout, had_stderr, new_stdout, new_stderr, new_exit, old_exit);
    // Ensure trailing newline
    if !result.ends_with('\n') {
        result.push('\n');
    }
    std::fs::write(path, result)?;
    Ok(())
}

fn rewrite_directives(
    content: &str,
    had_stdout: bool,
    had_stderr: bool,
    new_stdout: &str,
    new_stderr: &str,
    new_exit: i32,
    old_exit: i32,
) -> String {
    // Simple approach: strip old out:/err:/exit: directives and re-emit them
    let mut output = String::new();
    let mut skip = false;

    for line in content.lines() {
        let trimmed = line.trim();
        // Skip old stdout block or inline
        if trimmed.starts_with("// out:") || trimmed == "// :out" {
            if trimmed == "// out:" { skip = true; }
            if trimmed == "// :out" { skip = false; }
            continue;
        }
        // Skip old stderr block or inline
        if trimmed.starts_with("// err:") || trimmed == "// :err" {
            if trimmed == "// err:" { skip = true; }
            if trimmed == "// :err" { skip = false; }
            continue;
        }
        // Skip old exit:
        if trimmed.starts_with("// exit:") {
            continue;
        }
        if skip {
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }

    // Append new directives at the top (before source code)
    // Actually, insert them where the old ones were - this simple rewrite just appends at end
    // For a better implementation we'd track position, but for now prepend to output
    let mut new_directives = String::new();
    if had_stdout || !new_stdout.is_empty() {
        if new_stdout.contains('\n') {
            new_directives.push_str("// out:\n");
            for line in new_stdout.lines() {
                new_directives.push_str("// ");
                new_directives.push_str(line);
                new_directives.push('\n');
            }
            new_directives.push_str("// :out\n");
        } else {
            new_directives.push_str("// out: ");
            new_directives.push_str(new_stdout);
            new_directives.push('\n');
        }
    }
    if had_stderr || !new_stderr.is_empty() {
        if new_stderr.contains('\n') {
            new_directives.push_str("// err:\n");
            for line in new_stderr.lines() {
                new_directives.push_str("// ");
                new_directives.push_str(line);
                new_directives.push('\n');
            }
            new_directives.push_str("// :err\n");
        } else {
            new_directives.push_str("// err: ");
            new_directives.push_str(new_stderr);
            new_directives.push('\n');
        }
    }
    if new_exit != 0 || old_exit != 0 {
        new_directives.push_str(&format!("// exit: {new_exit}\n"));
    }

    new_directives + &output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_replaces_inline_out() {
        let content = "// out: old value\ncode here\n";
        let result = rewrite_directives(content, true, false, "new value", "", 0, 0);
        assert!(result.contains("// out: new value"), "got: {result}");
        assert!(!result.contains("old value"), "got: {result}");
    }
}
