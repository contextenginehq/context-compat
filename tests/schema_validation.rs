//! Schema validation tests: all outputs validate against JSON Schemas.

use context_compat::cli_runner::CliRunner;
use context_compat::fixture;
use jsonschema::validator_for;

fn cli() -> Option<CliRunner> {
    CliRunner::from_env()
}

fn validate(value: &serde_json::Value, schema_name: &str) {
    let schema = fixture::schema(schema_name);
    let validator = validator_for(&schema)
        .unwrap_or_else(|e| panic!("invalid schema '{schema_name}': {e}"));
    if let Err(e) = validator.validate(value) {
        panic!("output does not validate against '{schema_name}': {e}");
    }
}

/// Resolve output validates against selection_result schema.
#[test]
fn resolve_validates_schema() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
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
        let out = runner.resolve(&cache, &q.query, q.budget).unwrap();

        assert_eq!(
            out.exit_code, 0,
            "resolve failed for {cache_name}/{query_name}: {}",
            out.stderr
        );

        let value: serde_json::Value = serde_json::from_str(out.stdout.trim())
            .unwrap_or_else(|e| {
                panic!("invalid JSON from resolve {cache_name}/{query_name}: {e}")
            });

        validate(&value, "selection_result");
    }
}

/// Inspect output validates against inspect_output schema.
#[test]
fn inspect_validates_schema() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    for cache_name in &["minimal", "realistic"] {
        let cache = fixture::cache_path(cache_name);
        let out = runner.inspect(&cache).unwrap();

        assert_eq!(
            out.exit_code, 0,
            "inspect failed for {cache_name}: {}",
            out.stderr
        );

        let value: serde_json::Value = serde_json::from_str(out.stdout.trim())
            .unwrap_or_else(|e| panic!("invalid JSON from inspect {cache_name}: {e}"));

        validate(&value, "inspect_output");
    }
}

/// Inspect of an invalid cache still validates against inspect_output schema.
#[test]
fn inspect_invalid_validates_schema() {
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

    let out = runner.inspect(&cache).unwrap();

    // The CLI may return non-zero for invalid caches, but if it produces
    // JSON output, it should validate against the schema.
    if out.exit_code == 0 {
        let value: serde_json::Value = serde_json::from_str(out.stdout.trim()).unwrap();
        validate(&value, "inspect_output");
        assert_eq!(value["valid"], false, "corrupt cache should report valid=false");
    }
}

/// Build a fresh cache, then resolve — output validates against schema.
#[test]
fn freshly_built_cache_validates() {
    let runner = match cli() {
        Some(r) => r,
        None => {
            eprintln!("CONTEXT_CLI_BIN not set, skipping");
            return;
        }
    };

    let sources = fixture::documents_path("minimal");
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("fresh");

    let build_out = runner.build(&sources, &cache, false).unwrap();
    assert_eq!(build_out.exit_code, 0, "build failed: {}", build_out.stderr);

    // Inspect
    let inspect_out = runner.inspect(&cache).unwrap();
    assert_eq!(inspect_out.exit_code, 0);
    let inspect_val: serde_json::Value =
        serde_json::from_str(inspect_out.stdout.trim()).unwrap();
    validate(&inspect_val, "inspect_output");

    // Resolve
    let resolve_out = runner.resolve(&cache, "hello", 4000).unwrap();
    assert_eq!(resolve_out.exit_code, 0);
    let resolve_val: serde_json::Value =
        serde_json::from_str(resolve_out.stdout.trim()).unwrap();
    validate(&resolve_val, "selection_result");
}
