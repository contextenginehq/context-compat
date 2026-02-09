//! Backward compatibility tests: current binary reads pre-built v0 caches.
//! Also tests error boundaries: unsupported versions, IO failures, exit code contracts.

use context_compat::cli_runner::CliRunner;
use context_compat::fixture;

fn cli() -> Option<CliRunner> {
    CliRunner::from_env()
}

// --- Frozen CLI exit codes (per cli_spec.md) ---

const EXIT_SUCCESS: i32 = 0;
const EXIT_CACHE_MISSING: i32 = 4;
const EXIT_CACHE_INVALID: i32 = 5;
const EXIT_IO_ERROR: i32 = 6;

// --- v0 cache compatibility ---

/// Pre-built v0 minimal cache loads and produces valid resolve output.
#[test]
fn v0_cache_minimal_resolves() {
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

    assert_eq!(out.exit_code, EXIT_SUCCESS, "resolve failed on v0 cache: {}", out.stderr);

    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    assert!(v["documents"].is_array());
    assert!(v["selection"].is_object());
    assert_eq!(v["selection"]["documents_considered"], 2);
}

/// Pre-built v0 realistic cache loads and produces valid resolve output.
#[test]
fn v0_cache_realistic_resolves() {
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

    assert_eq!(out.exit_code, EXIT_SUCCESS, "resolve failed on v0 cache: {}", out.stderr);

    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    assert!(v["documents"].is_array());
    assert_eq!(v["selection"]["documents_considered"], 3);
}

/// Pre-built v0 caches report valid=true when inspected.
#[test]
fn v0_cache_inspect_valid() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    for name in &["minimal", "realistic"] {
        let cache = fixture::cache_path(name);
        let out = runner.inspect(&cache).unwrap();

        assert_eq!(out.exit_code, EXIT_SUCCESS, "inspect failed on v0 {name} cache: {}", out.stderr);

        let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
        assert_eq!(v["valid"], true, "v0 {name} cache should be valid");
    }
}

/// Resolving with all query fixtures against pre-built caches succeeds.
#[test]
fn v0_cache_all_queries() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let queries = &["basic", "zero_budget", "multi_term", "empty_query", "tight_budget"];

    for cache_name in &["minimal", "realistic"] {
        let cache = fixture::cache_path(cache_name);
        for query_name in queries {
            let q = fixture::query(query_name);
            let out = runner.resolve(&cache, &q.query, q.budget).unwrap();
            assert_eq!(
                out.exit_code, EXIT_SUCCESS,
                "resolve failed for {cache_name}/{query_name}: {}",
                out.stderr
            );
        }
    }
}

// --- Exit code contract tests ---

/// Missing cache path returns exit code 4 (CACHE_MISSING).
#[test]
fn exit_code_cache_missing() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nonexistent");

    let out = runner.inspect(&missing).unwrap();
    assert_eq!(
        out.exit_code, EXIT_CACHE_MISSING,
        "inspect of missing cache should return exit code {EXIT_CACHE_MISSING}, got {}",
        out.exit_code
    );

    let out = runner.resolve(&missing, "test", 100).unwrap();
    assert_eq!(
        out.exit_code, EXIT_CACHE_MISSING,
        "resolve of missing cache should return exit code {EXIT_CACHE_MISSING}, got {}",
        out.exit_code
    );
}

/// Corrupt manifest returns exit code 5 (CACHE_INVALID).
#[test]
fn exit_code_cache_invalid() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("corrupt");
    std::fs::create_dir(&cache).unwrap();
    std::fs::write(cache.join("manifest.json"), "not valid json").unwrap();

    let out = runner.resolve(&cache, "test", 100).unwrap();
    assert_eq!(
        out.exit_code, EXIT_CACHE_INVALID,
        "resolve of corrupt cache should return exit code {EXIT_CACHE_INVALID}, got {}",
        out.exit_code
    );
}

/// Resolving against a cache dir with no manifest at all returns CACHE_MISSING.
#[test]
fn exit_code_missing_manifest() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("no_manifest");
    std::fs::create_dir(&cache).unwrap();
    // Directory exists, but no manifest.json

    let out = runner.resolve(&cache, "test", 100).unwrap();
    // Missing manifest file inside an existing dir should still error
    assert_ne!(out.exit_code, EXIT_SUCCESS, "resolve should fail without manifest");
    // Could be CACHE_MISSING (file not found) or CACHE_INVALID
    assert!(
        out.exit_code == EXIT_CACHE_MISSING || out.exit_code == EXIT_CACHE_INVALID,
        "expected exit code {EXIT_CACHE_MISSING} or {EXIT_CACHE_INVALID}, got {}",
        out.exit_code
    );
}

// --- Permission denied / IO error boundary ---

/// Unreadable cache directory returns IO_ERROR or CACHE_MISSING.
#[test]
fn exit_code_permission_denied() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("unreadable");
    std::fs::create_dir(&cache).unwrap();
    std::fs::write(cache.join("manifest.json"), "{}").unwrap();

    // Make manifest unreadable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o000);
        std::fs::set_permissions(cache.join("manifest.json"), perms).unwrap();
    }

    #[cfg(unix)]
    {
        let out = runner.resolve(&cache, "test", 100).unwrap();
        assert_ne!(out.exit_code, EXIT_SUCCESS, "should fail on unreadable manifest");
        // IO_ERROR (6) for permission denied
        assert!(
            out.exit_code == EXIT_IO_ERROR || out.exit_code == EXIT_CACHE_MISSING,
            "expected exit code {EXIT_IO_ERROR} or {EXIT_CACHE_MISSING}, got {}",
            out.exit_code
        );

        // Restore permissions so tempdir cleanup works
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(cache.join("manifest.json"), perms).unwrap();
    }
}

// --- Unsupported cache version ---

/// A cache with build_config.version="999" should either be rejected or loaded.
/// This test documents the current behavior and will enforce rejection once
/// version validation is implemented.
#[test]
fn future_version_cache_behavior() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("future_version");

    let out = runner.resolve(&cache, "hello", 4000).unwrap();

    // Document current behavior: the binary may or may not reject future versions.
    // If it rejects: exit_code should be CACHE_INVALID (5).
    // If it accepts: the output should still be valid JSON.
    if out.exit_code == EXIT_SUCCESS {
        // Currently loads — verify output is at least valid JSON
        let v: serde_json::Value = serde_json::from_str(out.stdout.trim())
            .expect("future version cache should produce valid JSON if loaded");
        assert!(v["documents"].is_array());
    } else {
        // Version validation was added — verify it's the right error code
        assert_eq!(
            out.exit_code, EXIT_CACHE_INVALID,
            "future version rejection should use exit code {EXIT_CACHE_INVALID}, got {}",
            out.exit_code
        );
    }
}

/// Inspect of a future version cache should report the version info.
#[test]
fn future_version_inspect() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("future_version");
    let out = runner.inspect(&cache).unwrap();

    if out.exit_code == EXIT_SUCCESS {
        let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
        assert!(v["cache_version"].is_string());
        assert_eq!(v["document_count"], 1);
    }
}
