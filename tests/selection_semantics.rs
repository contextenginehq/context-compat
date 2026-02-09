//! Selection semantics tests: ordering stability, zero-score inclusion, float format.
//!
//! These lock the subtle behavioral contracts that are easy to break accidentally:
//! - Equal-score documents are tie-broken by document ID (ascending)
//! - Zero-score documents are included if budget allows
//! - Float formatting is stable across runs (serde_json minimal representation)

use context_compat::cli_runner::CliRunner;
use context_compat::fixture;

fn cli() -> Option<CliRunner> {
    CliRunner::from_env()
}

// --- Ordering stability under equal score ---

/// Two documents with equal scores must appear in document ID order (a.md before b.md).
/// This is the spec-mandated tie-breaking rule: (score DESC, id ASC).
#[test]
fn equal_score_ordered_by_id() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("tie_break");
    let q = fixture::query("tie_break");
    let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out.exit_code, 0, "resolve failed: {}", out.stderr);

    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    let docs = v["documents"].as_array().unwrap();

    assert_eq!(docs.len(), 2, "expected 2 documents");
    assert_eq!(docs[0]["id"], "a.md", "first document should be a.md");
    assert_eq!(docs[1]["id"], "b.md", "second document should be b.md");

    // Both must have identical scores
    assert_eq!(docs[0]["score"], docs[1]["score"], "scores should be equal");

    // Golden comparison locks the exact output
    let expected = fixture::expected("tie_break_ordering");
    let actual = fixture::canonicalize(&out.stdout);
    assert_eq!(actual, expected, "tie-break golden output mismatch");
}

// --- Zero-score inclusion ---

/// When no query terms match, all documents still get score 0.0 and are included
/// if budget allows. Order is by document ID (ascending) since all scores are equal.
#[test]
fn zero_score_documents_included() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let cache = fixture::cache_path("tie_break");
    let q = fixture::query("no_match");
    let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

    assert_eq!(out.exit_code, 0, "resolve failed: {}", out.stderr);

    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    let docs = v["documents"].as_array().unwrap();

    // All docs should be included despite zero score
    assert_eq!(docs.len(), 2, "zero-score docs should be included when budget allows");

    // All scores must be exactly 0.0
    for doc in docs {
        assert_eq!(
            doc["score"].as_f64().unwrap(),
            0.0,
            "non-matching doc {} should have score 0.0",
            doc["id"]
        );
    }

    // Order must be by ID
    assert_eq!(docs[0]["id"], "a.md");
    assert_eq!(docs[1]["id"], "b.md");

    // Selection metadata
    assert_eq!(v["selection"]["documents_selected"], 2);
    assert_eq!(v["selection"]["documents_excluded_by_budget"], 0);

    // Golden comparison
    let expected = fixture::expected("tie_break_zero_score");
    let actual = fixture::canonicalize(&out.stdout);
    assert_eq!(actual, expected, "zero-score golden output mismatch");
}

/// Zero budget excludes ALL documents, even zero-score ones.
#[test]
fn zero_budget_excludes_all() {
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

    let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
    let docs = v["documents"].as_array().unwrap();

    assert!(docs.is_empty(), "zero budget should produce empty documents array");
    assert_eq!(v["selection"]["documents_selected"], 0);
    assert!(
        v["selection"]["documents_excluded_by_budget"].as_u64().unwrap() > 0,
        "should report excluded documents"
    );
}

// --- Float format stability ---

/// Score serialization must use serde_json's minimal f32 representation.
/// This locks the exact byte format: 0.75 (not 0.750000), 0.5 (not 0.500000),
/// 0.33333334 (not 0.333333), 0.0 (not 0.00 or 0).
#[test]
fn float_format_stability() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    // Case 1: 3/4 = 0.75 (deployment.md in realistic with query "deployment")
    let cache = fixture::cache_path("realistic");
    let out = runner.resolve(&cache, "deployment", 4000).unwrap();
    assert_eq!(out.exit_code, 0);

    let raw = out.stdout.trim();

    // Check exact float representations in raw JSON string
    // deployment.md: 3 matches / 4 words = 0.75
    assert!(
        raw.contains("\"score\":0.75"),
        "expected score 0.75 in raw JSON, got: {raw}"
    );

    // empty.md: 0/0 = 0.0
    assert!(
        raw.contains("\"score\":0.0"),
        "expected score 0.0 in raw JSON, got: {raw}"
    );

    // Case 2: 1/3 = 0.33333334 (security.md with query "deployment security")
    let out2 = runner.resolve(&cache, "deployment security", 4000).unwrap();
    assert_eq!(out2.exit_code, 0);
    let raw2 = out2.stdout.trim();

    assert!(
        raw2.contains("\"score\":0.33333334"),
        "expected score 0.33333334 (f32 representation of 1/3) in raw JSON, got: {raw2}"
    );

    // Case 3: 1/2 = 0.5 (tie_break docs)
    let cache_tb = fixture::cache_path("tie_break");
    let out3 = runner.resolve(&cache_tb, "deployment", 4000).unwrap();
    assert_eq!(out3.exit_code, 0);
    let raw3 = out3.stdout.trim();

    assert!(
        raw3.contains("\"score\":0.5"),
        "expected score 0.5 in raw JSON, got: {raw3}"
    );
}
