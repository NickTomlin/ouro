use colored::Colorize;
use similar::{ChangeTag, TextDiff};

pub fn print_diff(stream: &str, expected: &str, actual: &str) {
    eprintln!("\n--- {stream} ---");
    let diff = TextDiff::from_lines(expected, actual);
    for change in diff.iter_all_changes() {
        let line = change.value();
        match change.tag() {
            ChangeTag::Delete => eprint!("{}", format!("- {line}").red()),
            ChangeTag::Insert => eprint!("{}", format!("+ {line}").green()),
            ChangeTag::Equal => eprint!("  {line}"),
        }
    }
}

pub fn print_exit_mismatch(expected: i32, actual: i32) {
    eprintln!(
        "\n--- exit code ---\n{}\n{}",
        format!("- {expected}").red(),
        format!("+ {actual}").green()
    );
}
