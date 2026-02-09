use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A query fixture loaded from `fixtures/v0/queries/*.json`.
#[derive(Debug, Deserialize)]
pub struct QueryFixture {
    pub query: String,
    pub budget: usize,
}

/// Root directory for all fixtures: `CARGO_MANIFEST_DIR/fixtures`.
pub fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Root directory for v0 contract fixtures: `fixtures/v0`.
pub fn v0_root() -> PathBuf {
    fixtures_root().join("v0")
}

/// Path to a pre-built v0 cache: `fixtures/v0/caches/{name}`.
pub fn cache_path(name: &str) -> PathBuf {
    v0_root().join("caches").join(name)
}

/// Path to a document fixture directory: `fixtures/v0/documents/{name}`.
pub fn documents_path(name: &str) -> PathBuf {
    v0_root().join("documents").join(name)
}

/// Load and parse a query fixture from `fixtures/v0/queries/{name}.json`.
pub fn query(name: &str) -> QueryFixture {
    let path = v0_root().join("queries").join(format!("{name}.json"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read query fixture {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse query fixture {}: {e}", path.display()))
}

/// Load an expected output file from `fixtures/v0/expected/{name}.json`,
/// canonicalized for cross-platform comparison.
pub fn expected(name: &str) -> String {
    let path = v0_root().join("expected").join(format!("{name}.json"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read expected fixture {}: {e}", path.display()));
    canonicalize(&content)
}

/// Root directory for schemas: `CARGO_MANIFEST_DIR/schemas`.
pub fn schemas_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

/// Load a JSON Schema from `schemas/{name}.schema.json` as a `serde_json::Value`.
pub fn schema(name: &str) -> serde_json::Value {
    let path = schemas_root().join(format!("{name}.schema.json"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read schema {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse schema {}: {e}", path.display()))
}

/// Canonicalize output for cross-platform comparison.
///
/// - Normalizes CRLF → LF
/// - Trims trailing whitespace per line
/// - Strips trailing newlines
pub fn canonicalize(output: &str) -> String {
    output
        .replace("\r\n", "\n")
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}
