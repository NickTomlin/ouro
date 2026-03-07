use colored::Colorize;
use similar::{ChangeTag, TextDiff};

pub fn print_diff(stream: &str, expected: &str, actual: &str) {
    eprintln!("  {stream}:");
    let diff = TextDiff::from_lines(expected, actual);
    for (i, group) in diff.grouped_ops(2).iter().enumerate() {
        if i > 0 {
            eprintln!("{}", "      ...".dimmed());
        }
        for op in group {
            for change in diff.iter_changes(op) {
                let line = change.value();
                match change.tag() {
                    ChangeTag::Delete => eprint!("{}", format!("    - {line}").red()),
                    ChangeTag::Insert => eprint!("{}", format!("    + {line}").green()),
                    ChangeTag::Equal => eprint!("{}", format!("      {line}").dimmed()),
                }
            }
        }
    }
    eprintln!();
}

pub fn print_exit_mismatch(expected: i32, actual: i32) {
    eprintln!(
        "  exit code: {} → {}",
        format!("{expected}").red(),
        format!("{actual}").green()
    );
    eprintln!();
}
