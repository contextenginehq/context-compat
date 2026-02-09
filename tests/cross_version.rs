//! Cross-version regression tests: compare current binary output against previous binary.
//!
//! Requires `CONTEXT_PREV_BIN` env var pointing to a previous release binary.
//! Skips gracefully if not set.
//!
//! This catches drift even when golden fixtures were accidentally regenerated.

use context_compat::cli_runner::CliRunner;
use context_compat::fixture;

fn current() -> Option<CliRunner> {
    CliRunner::from_env()
}

fn previous() -> Option<CliRunner> {
    std::env::var("CONTEXT_PREV_BIN")
        .ok()
        .map(|p| CliRunner::new(p))
}

/// Resolve output from current and previous binaries must be byte-identical.
#[test]
fn resolve_matches_previous_binary() {
    let curr = match current() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };
    let prev = match previous() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_PREV_BIN not set, skipping");
            return;
        }
    };

    let cases = [
        ("minimal", "basic"),
        ("minimal", "zero_budget"),
        ("realistic", "basic"),
        ("realistic", "multi_term"),
        ("realistic", "empty_query"),
        ("minimal", "tight_budget"),
    ];

    for (cache_name, query_name) in &cases {
        let cache = fixture::cache_path(cache_name);
        let q = fixture::query(query_name);

        let curr_out = curr.resolve(&cache, &q.query, q.budget).unwrap();
        let prev_out = prev.resolve(&cache, &q.query, q.budget).unwrap();

        assert_eq!(
            curr_out.exit_code, 0,
            "current binary failed on {cache_name}/{query_name}: {}",
            curr_out.stderr
        );
        assert_eq!(
            prev_out.exit_code, 0,
            "previous binary failed on {cache_name}/{query_name}: {}",
            prev_out.stderr
        );

        let curr_canon = fixture::canonicalize(&curr_out.stdout);
        let prev_canon = fixture::canonicalize(&prev_out.stdout);

        assert_eq!(
            curr_canon, prev_canon,
            "resolve output differs between current and previous binary for {cache_name}/{query_name}"
        );
    }
}

/// Inspect output from current and previous binaries must match on key fields.
#[test]
fn inspect_matches_previous_binary() {
    let curr = match current() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };
    let prev = match previous() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_PREV_BIN not set, skipping");
            return;
        }
    };

    for cache_name in &["minimal", "realistic"] {
        let cache = fixture::cache_path(cache_name);

        let curr_out = curr.inspect(&cache).unwrap();
        let prev_out = prev.inspect(&cache).unwrap();

        assert_eq!(curr_out.exit_code, 0);
        assert_eq!(prev_out.exit_code, 0);

        let curr_val: serde_json::Value =
            serde_json::from_str(curr_out.stdout.trim()).unwrap();
        let prev_val: serde_json::Value =
            serde_json::from_str(prev_out.stdout.trim()).unwrap();

        assert_eq!(
            curr_val["cache_version"], prev_val["cache_version"],
            "cache_version differs for {cache_name}"
        );
        assert_eq!(
            curr_val["document_count"], prev_val["document_count"],
            "document_count differs for {cache_name}"
        );
        assert_eq!(
            curr_val["valid"], prev_val["valid"],
            "valid differs for {cache_name}"
        );
    }
}

/// Build from the same sources with both binaries — cache versions must match.
#[test]
fn build_version_matches_previous_binary() {
    let curr = match current() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };
    let prev = match previous() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_PREV_BIN not set, skipping");
            return;
        }
    };

    let sources = fixture::documents_path("minimal");
    let dir = tempfile::tempdir().unwrap();
    let cache_curr = dir.path().join("curr");
    let cache_prev = dir.path().join("prev");

    let bc = curr.build(&sources, &cache_curr, false).unwrap();
    let bp = prev.build(&sources, &cache_prev, false).unwrap();

    assert_eq!(bc.exit_code, 0, "current build failed: {}", bc.stderr);
    assert_eq!(bp.exit_code, 0, "previous build failed: {}", bp.stderr);

    let ic = curr.inspect(&cache_curr).unwrap();
    let ip = prev.inspect(&cache_prev).unwrap();

    let vc: serde_json::Value = serde_json::from_str(ic.stdout.trim()).unwrap();
    let vp: serde_json::Value = serde_json::from_str(ip.stdout.trim()).unwrap();

    assert_eq!(
        vc["cache_version"], vp["cache_version"],
        "cache version from current and previous binary differ"
    );
}
