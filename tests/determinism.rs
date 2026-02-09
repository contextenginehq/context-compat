//! Determinism tests: same input twice produces byte-identical output.

use context_compat::cli_runner::CliRunner;
use context_compat::fixture;

fn cli() -> Option<CliRunner> {
    CliRunner::from_env()
}

/// Run resolve twice with the same inputs — stdout must be byte-identical.
#[test]
fn resolve_deterministic_minimal_basic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("minimal");
    let q = fixture::query("basic");

    let out1 = runner.resolve(&cache, &q.query, q.budget).unwrap();
    let out2 = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out1.exit_code, 0, "first run failed: {}", out1.stderr);
    assert_eq!(out2.exit_code, 0, "second run failed: {}", out2.stderr);
    assert_eq!(
        out1.stdout, out2.stdout,
        "resolve output is not deterministic"
    );
}

#[test]
fn resolve_deterministic_realistic_basic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("realistic");
    let q = fixture::query("basic");

    let out1 = runner.resolve(&cache, &q.query, q.budget).unwrap();
    let out2 = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out1.exit_code, 0);
    assert_eq!(out2.exit_code, 0);
    assert_eq!(
        out1.stdout, out2.stdout,
        "resolve output is not deterministic"
    );
}

#[test]
fn resolve_deterministic_realistic_multi_term() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("realistic");
    let q = fixture::query("multi_term");

    let out1 = runner.resolve(&cache, &q.query, q.budget).unwrap();
    let out2 = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out1.exit_code, 0);
    assert_eq!(out2.exit_code, 0);
    assert_eq!(
        out1.stdout, out2.stdout,
        "resolve output is not deterministic"
    );
}

#[test]
fn resolve_deterministic_zero_budget() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("minimal");
    let q = fixture::query("zero_budget");

    let out1 = runner.resolve(&cache, &q.query, q.budget).unwrap();
    let out2 = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out1.exit_code, 0);
    assert_eq!(out2.exit_code, 0);
    assert_eq!(
        out1.stdout, out2.stdout,
        "resolve output is not deterministic"
    );
}

/// Build a cache twice from the same sources in different directories.
/// Verifies path independence: manifest hashes, document file hashes, and
/// resolve output must all be identical between the two builds.
#[test]
fn build_deterministic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let sources = fixture::documents_path("minimal");
    let dir = tempfile::tempdir().unwrap();
    let cache1 = dir.path().join("cache1");
    let cache2 = dir.path().join("cache2");

    let b1 = runner.build(&sources, &cache1, false).unwrap();
    let b2 = runner.build(&sources, &cache2, false).unwrap();

    assert_eq!(b1.exit_code, 0, "first build failed: {}", b1.stderr);
    assert_eq!(b2.exit_code, 0, "second build failed: {}", b2.stderr);

    // 1. Inspect output: cache_version, document_count, valid must match
    let i1 = runner.inspect(&cache1).unwrap();
    let i2 = runner.inspect(&cache2).unwrap();

    assert_eq!(i1.exit_code, 0);
    assert_eq!(i2.exit_code, 0);

    let v1: serde_json::Value = serde_json::from_str(i1.stdout.trim()).unwrap();
    let v2: serde_json::Value = serde_json::from_str(i2.stdout.trim()).unwrap();

    assert_eq!(v1["cache_version"], v2["cache_version"], "cache_version differs");
    assert_eq!(v1["document_count"], v2["document_count"], "document_count differs");
    assert_eq!(v1["valid"], v2["valid"], "valid differs");

    // 2. Manifest comparison: all fields except created_at must be identical
    let m1: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(cache1.join("manifest.json")).unwrap(),
    )
    .unwrap();
    let m2: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(cache2.join("manifest.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(m1["cache_version"], m2["cache_version"], "manifest cache_version differs");
    assert_eq!(m1["build_config"], m2["build_config"], "manifest build_config differs");
    assert_eq!(m1["document_count"], m2["document_count"], "manifest document_count differs");
    assert_eq!(m1["documents"], m2["documents"], "manifest documents array differs");

    // 3. Index file must be byte-identical
    let idx1 = std::fs::read_to_string(cache1.join("index.json")).unwrap();
    let idx2 = std::fs::read_to_string(cache2.join("index.json")).unwrap();
    assert_eq!(idx1, idx2, "index.json differs");

    // 4. Each document file must be byte-identical
    let docs = m1["documents"].as_array().unwrap();
    for doc in docs {
        let file = doc["file"].as_str().unwrap();
        let d1 = std::fs::read(cache1.join(file)).unwrap();
        let d2 = std::fs::read(cache2.join(file)).unwrap();
        assert_eq!(d1, d2, "document file {} differs", file);
    }

    // 5. Resolve against both caches should produce identical output
    let r1 = runner.resolve(&cache1, "hello", 4000).unwrap();
    let r2 = runner.resolve(&cache2, "hello", 4000).unwrap();

    assert_eq!(r1.exit_code, 0);
    assert_eq!(r2.exit_code, 0);
    assert_eq!(
        r1.stdout, r2.stdout,
        "resolve against rebuilt caches is not deterministic"
    );
}

/// Build from realistic sources also produces deterministic output.
#[test]
fn build_deterministic_realistic() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let sources = fixture::documents_path("realistic");
    let dir = tempfile::tempdir().unwrap();
    let cache1 = dir.path().join("cache1");
    let cache2 = dir.path().join("cache2");

    let b1 = runner.build(&sources, &cache1, false).unwrap();
    let b2 = runner.build(&sources, &cache2, false).unwrap();

    assert_eq!(b1.exit_code, 0, "first build failed: {}", b1.stderr);
    assert_eq!(b2.exit_code, 0, "second build failed: {}", b2.stderr);

    // Manifest documents array must be identical (verifies deterministic sort order)
    let m1: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(cache1.join("manifest.json")).unwrap(),
    )
    .unwrap();
    let m2: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(cache2.join("manifest.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(m1["cache_version"], m2["cache_version"]);
    assert_eq!(m1["documents"], m2["documents"]);
}
