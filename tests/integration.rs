/// Integration tests: run the golden test suite against the fake compiler.

fn example_binary() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir).join("tests/fixtures/example")
}

#[test]
fn golden_suite_passes() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    ouro::Suite::new()
        .binary(example_binary())
        .files(format!("{manifest_dir}/tests/golden/*.example"))
        .prefix("// ")
        .run()
        .expect("golden test suite should pass");
}

#[test]
fn files_directory_expansion() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    ouro::Suite::new()
        .binary(example_binary())
        .files(format!("{manifest_dir}/tests/golden"))
        .prefix("// ")
        .run()
        .expect("directory path should expand to its contents");
}

#[test]
fn files_multiple_patterns() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    ouro::Suite::new()
        .binary(example_binary())
        .files_from_vec(vec![
            format!("{manifest_dir}/tests/golden/simple.example"),
            format!("{manifest_dir}/tests/golden/errors.example"),
        ])
        .prefix("// ")
        .run()
        .expect("multiple file patterns should all be collected");
}

#[test]
fn golden_suite_detects_failure() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a test file with wrong expected output
    let mut f = NamedTempFile::with_suffix(".example").unwrap();
    writeln!(f, r#"// out: wrong expected"#).unwrap();
    writeln!(f, r#"print "actual output""#).unwrap();
    let path = f.path().to_path_buf();

    let result = ouro::Suite::new()
        .binary(example_binary())
        .files(path.to_str().unwrap())
        .prefix("// ")
        .run();

    assert!(result.is_err(), "suite should fail on mismatched output");
}
