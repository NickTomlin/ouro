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

/// Update the test file in-place: rewrite expected stdout/stderr/exit with actual output.
pub fn update_test(tc: &TestCase, binary: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(binary)
        .args(&tc.args)
        .arg(&tc.path)
        .output()?;

    let actual_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let actual_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let actual_exit = output.status.code().unwrap_or(-1);

    let new_stdout = actual_stdout.trim_end_matches('\n');
    let new_stderr = actual_stderr.trim_end_matches('\n');

    // Write a stdout directive if there was one before, or actual output is non-empty (new snapshot)
    let stdout_directive = if tc.expected_stdout.is_some() || !new_stdout.is_empty() {
        Some(new_stdout)
    } else {
        None
    };

    let stderr_directive = if tc.expected_stderr.is_some() || !new_stderr.is_empty() {
        Some(new_stderr)
    } else {
        None
    };

    // Keep exit directive if new or old value is non-zero
    let write_exit = actual_exit != 0 || tc.expected_exit != 0;

    let content = std::fs::read_to_string(&tc.path)?;
    let mut result = rewrite_directives(
        &content,
        &tc.prefix,
        stdout_directive,
        stderr_directive,
        actual_exit,
        write_exit,
    );
    if !result.ends_with('\n') {
        result.push('\n');
    }
    std::fs::write(&tc.path, result)?;
    Ok(())
}

/// Single-pass rewriter: strips all out:/err:/exit: directives, tracks their first position,
/// then reinserts new values at those positions.
fn rewrite_directives(
    content: &str,
    prefix: &str,
    new_stdout: Option<&str>,
    new_stderr: Option<&str>,
    new_exit: i32,
    write_exit: bool,
) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    let mut in_stdout_block = false;
    let mut in_stderr_block = false;
    let mut stdout_insertion: Option<usize> = None;
    let mut stderr_insertion: Option<usize> = None;
    let mut exit_insertion: Option<usize> = None;
    let mut first_source_line: Option<usize> = None;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(prefix) {
            let trimmed = rest.trim();

            if in_stdout_block {
                if trimmed == ":out" {
                    in_stdout_block = false;
                }
                continue;
            }
            if in_stderr_block {
                if trimmed == ":err" {
                    in_stderr_block = false;
                }
                continue;
            }

            // Block open (exact match: nothing after keyword)
            if trimmed == "out:" {
                stdout_insertion.get_or_insert(out_lines.len());
                in_stdout_block = true;
                continue;
            }
            if trimmed == "err:" {
                stderr_insertion.get_or_insert(out_lines.len());
                in_stderr_block = true;
                continue;
            }

            // Inline or stray close tags
            if trimmed.starts_with("out:") || trimmed == ":out" {
                stdout_insertion.get_or_insert(out_lines.len());
                continue;
            }
            if trimmed.starts_with("err:") || trimmed == ":err" {
                stderr_insertion.get_or_insert(out_lines.len());
                continue;
            }
            if trimmed.starts_with("exit:") {
                exit_insertion.get_or_insert(out_lines.len());
                continue;
            }

            // Other prefix line (args, unknown) — keep it
            out_lines.push(format!("{line}\n"));
        } else {
            // Source line
            first_source_line.get_or_insert(out_lines.len());
            out_lines.push(format!("{line}\n"));
        }
    }

    // Fallback insertion point: before first source line, or end of file
    let fallback = first_source_line.unwrap_or(out_lines.len());

    let mut insertions: Vec<(usize, String)> = Vec::new();

    if let Some(stdout) = new_stdout {
        let pos = stdout_insertion.unwrap_or(fallback);
        insertions.push((pos, format_directive(prefix, "out", stdout)));
    }
    if let Some(stderr) = new_stderr {
        let pos = stderr_insertion.unwrap_or(fallback);
        insertions.push((pos, format_directive(prefix, "err", stderr)));
    }
    if write_exit {
        let pos = exit_insertion.unwrap_or(fallback);
        insertions.push((pos, format!("{prefix}exit: {new_exit}\n")));
    }

    // Stable sort by position so relative order of same-position insertions is preserved
    insertions.sort_by_key(|(pos, _)| *pos);

    let mut result = String::new();
    let mut insert_idx = 0;

    for (line_idx, line) in out_lines.iter().enumerate() {
        while insert_idx < insertions.len() && insertions[insert_idx].0 == line_idx {
            result.push_str(&insertions[insert_idx].1);
            insert_idx += 1;
        }
        result.push_str(line);
    }
    while insert_idx < insertions.len() {
        result.push_str(&insertions[insert_idx].1);
        insert_idx += 1;
    }

    result
}

fn format_directive(prefix: &str, keyword: &str, content: &str) -> String {
    if content.contains('\n') {
        let mut s = format!("{prefix}{keyword}:\n");
        for line in content.lines() {
            s.push_str(prefix);
            s.push_str(line);
            s.push('\n');
        }
        s.push_str(prefix);
        s.push(':');
        s.push_str(keyword);
        s.push('\n');
        s
    } else {
        format!("{prefix}{keyword}: {content}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite(content: &str, stdout: Option<&str>, stderr: Option<&str>, exit: i32, write_exit: bool) -> String {
        rewrite_directives(content, "// ", stdout, stderr, exit, write_exit)
    }

    #[test]
    fn rewrite_replaces_inline_out() {
        let content = "// out: old value\ncode here\n";
        let result = rewrite(content, Some("new value"), None, 0, false);
        assert!(result.contains("// out: new value"), "got: {result}");
        assert!(!result.contains("old value"), "got: {result}");
    }

    #[test]
    fn rewrite_preserves_args_position() {
        let content = "// args: --foo\n// out: old\ncode here\n";
        let result = rewrite(content, Some("new"), None, 0, false);
        let args_pos = result.find("// args:").unwrap();
        let out_pos = result.find("// out:").unwrap();
        assert!(args_pos < out_pos, "args should precede out:\n{result}");
        assert!(result.contains("// args: --foo"), "args preserved: {result}");
        assert!(result.contains("// out: new"), "out updated: {result}");
        assert!(!result.contains("old"), "old value removed: {result}");
    }

    #[test]
    fn rewrite_collapses_stacked_directives() {
        let content = "// out: first\n// out: second\ncode here\n";
        let result = rewrite(content, Some("new value"), None, 0, false);
        let count = result.matches("// out:").count();
        assert_eq!(count, 1, "expected exactly one out: directive, got:\n{result}");
        assert!(result.contains("// out: new value"), "got: {result}");
    }

    #[test]
    fn rewrite_updates_stderr() {
        let content = "// err: old error\ncode here\n";
        let result = rewrite(content, None, Some("new error"), 0, false);
        assert!(result.contains("// err: new error"), "got: {result}");
        assert!(!result.contains("old error"), "got: {result}");
    }

    #[test]
    fn rewrite_new_snapshot_inserts_before_first_source_line() {
        let content = "// args: --foo\ncode here\n";
        let result = rewrite(content, Some("output"), None, 0, false);
        let args_pos = result.find("// args:").unwrap();
        let out_pos = result.find("// out:").unwrap();
        let code_pos = result.find("code here").unwrap();
        assert!(args_pos < out_pos, "args before out:\n{result}");
        assert!(out_pos < code_pos, "out: before source code:\n{result}");
    }

    #[test]
    fn rewrite_multiline_uses_block_form() {
        let content = "// out: old\ncode here\n";
        let result = rewrite(content, Some("line1\nline2"), None, 0, false);
        assert!(result.contains("// out:\n"), "block open: {result}");
        assert!(result.contains("// line1\n"), "line1: {result}");
        assert!(result.contains("// line2\n"), "line2: {result}");
        assert!(result.contains("// :out\n"), "block close: {result}");
    }

    #[test]
    fn rewrite_strips_block_out_directives() {
        let content = "// out:\n// line one\n// line two\n// :out\ncode here\n";
        let result = rewrite(content, Some("new value"), None, 0, false);
        assert!(result.contains("// out: new value"), "got: {result}");
        assert!(!result.contains("line one"), "old content removed: {result}");
        assert!(!result.contains("line two"), "old content removed: {result}");
        assert!(!result.contains(":out"), "block close removed: {result}");
    }

    #[test]
    fn rewrite_exit_nonzero() {
        let content = "// exit: 1\ncode here\n";
        let result = rewrite(content, None, None, 42, true);
        assert!(result.contains("// exit: 42"), "got: {result}");
    }

    #[test]
    fn rewrite_clears_exit_when_now_zero() {
        let content = "// exit: 1\ncode here\n";
        let result = rewrite(content, None, None, 0, true);
        assert!(result.contains("// exit: 0"), "got: {result}");
    }
}
