use std::path::{Path, PathBuf};
use crate::patterns::PatternSet;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub path: PathBuf,
    pub prefix: String,
    pub args: Vec<String>,
    pub expected_stdout: Option<String>,
    pub expected_stderr: Option<String>,
    pub expected_exit: i32,
}

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    UnclosedBlock { file: PathBuf, kind: &'static str },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {e}"),
            ParseError::UnclosedBlock { file, kind } => {
                write!(f, "{}: unclosed {kind} block", file.display())
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::Io(e)
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    InArgs,
    InStdout,
    InStderr,
}

fn append_to(slot: &mut Option<String>, value: &str) {
    match slot {
        Some(ref mut s) => { s.push('\n'); s.push_str(value); }
        None => *slot = Some(value.to_string()),
    }
}

pub fn parse_file(path: &Path, patterns: &dyn PatternSet) -> Result<TestCase, ParseError> {
    let content = std::fs::read_to_string(path)?;
    let prefix = patterns.prefix();

    let mut args_parts: Vec<String> = Vec::new();
    let mut stdout_lines: Vec<String> = Vec::new();
    let mut stderr_lines: Vec<String> = Vec::new();
    let mut expected_stdout: Option<String> = None;
    let mut expected_stderr: Option<String> = None;
    let mut expected_exit: i32 = 0;
    let mut state = State::Idle;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(prefix) {
            let trimmed = rest.trim();

            match &state {
                State::InArgs => {
                    if trimmed == patterns.args_close() {
                        // close block — already accumulated in args_parts
                        state = State::Idle;
                    } else {
                        let arg = rest.trim().to_string();
                        if !arg.is_empty() {
                            args_parts.push(arg);
                        }
                    }
                    continue;
                }
                State::InStdout => {
                    if trimmed == patterns.stdout_close() {
                        append_to(&mut expected_stdout, &stdout_lines.join("\n"));
                        stdout_lines.clear();
                        state = State::Idle;
                    } else {
                        stdout_lines.push(rest.to_string());
                    }
                    continue;
                }
                State::InStderr => {
                    if trimmed == patterns.stderr_close() {
                        append_to(&mut expected_stderr, &stderr_lines.join("\n"));
                        stderr_lines.clear();
                        state = State::Idle;
                    } else {
                        stderr_lines.push(rest.to_string());
                    }
                    continue;
                }
                State::Idle => {}
            }

            // In Idle: classify the directive
            if trimmed == patterns.args_open() {
                state = State::InArgs;
            } else if trimmed == patterns.stdout_open() {
                state = State::InStdout;
            } else if trimmed == patterns.stderr_open() {
                state = State::InStderr;
            } else if let Some(val) = trimmed.strip_prefix(patterns.args_inline()) {
                // inline args: accumulate
                let val = val.trim();
                if !val.is_empty() {
                    if let Some(parts) = shlex::split(val) {
                        args_parts.extend(parts);
                    }
                }
            } else if let Some(val) = trimmed.strip_prefix(patterns.stdout_inline()) {
                append_to(&mut expected_stdout, val.trim());
            } else if let Some(val) = trimmed.strip_prefix(patterns.stderr_inline()) {
                append_to(&mut expected_stderr, val.trim());
            } else if let Some(val) = trimmed.strip_prefix(patterns.exit()) {
                let val = val.trim();
                if let Ok(n) = val.parse::<i32>() {
                    expected_exit = n;
                }
            }
            // else: unrecognized directive line — ignore
        } else {
            // Line doesn't start with prefix
            if state != State::Idle {
                return Err(ParseError::UnclosedBlock {
                    file: path.to_path_buf(),
                    kind: match state {
                        State::InArgs => "args",
                        State::InStdout => "out",
                        State::InStderr => "err",
                        State::Idle => unreachable!(),
                    },
                });
            }
        }
    }

    // Check for unclosed block at EOF
    if state != State::Idle {
        return Err(ParseError::UnclosedBlock {
            file: path.to_path_buf(),
            kind: match state {
                State::InArgs => "args",
                State::InStdout => "out",
                State::InStderr => "err",
                State::Idle => unreachable!(),
            },
        });
    }

    Ok(TestCase {
        path: path.to_path_buf(),
        prefix: prefix.to_string(),
        args: args_parts,
        expected_stdout,
        expected_stderr,
        expected_exit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::DefaultPatterns;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn patterns() -> DefaultPatterns {
        DefaultPatterns::new("// ")
    }

    fn parse(content: &str) -> Result<TestCase, ParseError> {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        parse_file(f.path(), &patterns())
    }

    #[test]
    fn test_inline_out() {
        let tc = parse("// out: hello world\n").unwrap();
        assert_eq!(tc.expected_stdout, Some("hello world".to_string()));
    }

    #[test]
    fn test_block_out() {
        let tc = parse("// out:\n// line one\n// line two\n// :out\n").unwrap();
        assert_eq!(tc.expected_stdout, Some("line one\nline two".to_string()));
    }

    #[test]
    fn test_inline_args() {
        let tc = parse("// args: --foo bar\n").unwrap();
        assert_eq!(tc.args, vec!["--foo", "bar"]);
    }

    #[test]
    fn test_inline_err() {
        let tc = parse("// err: oops\n").unwrap();
        assert_eq!(tc.expected_stderr, Some("oops".to_string()));
    }

    #[test]
    fn test_exit_code() {
        let tc = parse("// exit: 42\n").unwrap();
        assert_eq!(tc.expected_exit, 42);
    }

    #[test]
    fn test_block_with_braces_in_content() {
        let content = "// out:\n// { foo }\n// :out\n";
        let tc = parse(content).unwrap();
        assert_eq!(tc.expected_stdout, Some("{ foo }".to_string()));
    }

    #[test]
    fn test_non_prefix_line_in_idle_ok() {
        let tc = parse("let x = 42;\n// out: 42\n").unwrap();
        assert_eq!(tc.expected_stdout, Some("42".to_string()));
    }

    #[test]
    fn test_stacked_inline_out() {
        let tc = parse("// out: foo\n// out: bar\n").unwrap();
        assert_eq!(tc.expected_stdout, Some("foo\nbar".to_string()));
    }

    #[test]
    fn test_stacked_inline_then_block() {
        let content = "// out: foo\n// out:\n// bar\n// baz\n// :out\n";
        let tc = parse(content).unwrap();
        assert_eq!(tc.expected_stdout, Some("foo\nbar\nbaz".to_string()));
    }

    #[test]
    fn test_stacked_block_then_inline() {
        let content = "// out:\n// foo\n// bar\n// :out\n// out: baz\n";
        let tc = parse(content).unwrap();
        assert_eq!(tc.expected_stdout, Some("foo\nbar\nbaz".to_string()));
    }

    #[test]
    fn test_stacked_inline_err() {
        let tc = parse("// err: oops\n// err: also bad\n").unwrap();
        assert_eq!(tc.expected_stderr, Some("oops\nalso bad".to_string()));
    }

    #[test]
    fn test_unclosed_block_is_error() {
        let result = parse("// out:\n// some line\ncode here\n");
        assert!(matches!(result, Err(ParseError::UnclosedBlock { .. })));
    }
}
