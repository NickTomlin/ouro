/// Integration tests: run the golden test suite against the fake compiler.

#[test]
fn golden_suite_passes() {
    // Find the ouro.toml relative to the Cargo.toml/manifest dir
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    ouro::Suite::new()
        .binary(std::path::Path::new(manifest_dir).join("tests/fixtures/myc"))
        .files(format!("{manifest_dir}/tests/golden/*.myc"))
        .prefix("// ")
        .run()
        .expect("golden test suite should pass");
}

#[test]
fn golden_suite_detects_failure() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a test file with wrong expected output
    let mut f = NamedTempFile::with_suffix(".myc").unwrap();
    writeln!(f, r#"// out: wrong expected"#).unwrap();
    writeln!(f, r#"print "actual output""#).unwrap();
    let path = f.path().to_path_buf();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let result = ouro::Suite::new()
        .binary(std::path::Path::new(manifest_dir).join("tests/fixtures/myc"))
        .files(path.to_str().unwrap())
        .prefix("// ")
        .run();

    assert!(result.is_err(), "suite should fail on mismatched output");
}
