//! Golden output tests: output matches committed expected/ files.

use context_compat::cli_runner::CliRunner;
use context_compat::fixture;

fn cli() -> Option<CliRunner> {
    CliRunner::from_env()
}

/// Helper to compare CLI output to an expected fixture using canonical comparison.
fn assert_golden(actual_stdout: &str, expected_name: &str) {
    let expected = fixture::expected(expected_name);
    let actual = fixture::canonicalize(actual_stdout);
    assert_eq!(
        actual, expected,
        "golden output mismatch for '{expected_name}'"
    );
}

#[test]
fn golden_minimal_basic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("minimal");
    let q = fixture::query("basic");
    let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out.exit_code, 0, "resolve failed: {}", out.stderr);
    assert_golden(&out.stdout, "minimal_basic");
}

#[test]
fn golden_minimal_zero_budget() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("minimal");
    let q = fixture::query("zero_budget");
    let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out.exit_code, 0, "resolve failed: {}", out.stderr);
    assert_golden(&out.stdout, "minimal_zero_budget");
}

#[test]
fn golden_realistic_basic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("realistic");
    let q = fixture::query("basic");
    let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out.exit_code, 0, "resolve failed: {}", out.stderr);
    assert_golden(&out.stdout, "realistic_basic");
}

#[test]
fn golden_realistic_multi_term() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("realistic");
    let q = fixture::query("multi_term");
    let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out.exit_code, 0, "resolve failed: {}", out.stderr);
    assert_golden(&out.stdout, "realistic_multi_term");
}

#[test]
fn golden_inspect_minimal() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("minimal");
    let out = runner.inspect(&cache).unwrap();

    assert_eq!(out.exit_code, 0, "inspect failed: {}", out.stderr);
    assert_golden(&out.stdout, "inspect_minimal");
}

#[test]
fn golden_inspect_realistic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("realistic");
    let out = runner.inspect(&cache).unwrap();

    assert_eq!(out.exit_code, 0, "inspect failed: {}", out.stderr);
    assert_golden(&out.stdout, "inspect_realistic");
}
