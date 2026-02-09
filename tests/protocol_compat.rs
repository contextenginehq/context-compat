//! MCP protocol compatibility tests: server responds correctly to JSON-RPC requests.
//! Includes sequential stability (concurrency sanity) tests.

use context_compat::fixture;
use context_compat::mcp_runner::McpRunner;

fn mcp(cache_root: &std::path::Path) -> Option<McpRunner> {
    match McpRunner::from_env(cache_root) {
        Some(Ok(runner)) => Some(runner),
        Some(Err(e)) => panic!("failed to spawn MCP server: {e}"),
        None => {
            eprintln!("MCP_SERVER_BIN not set, skipping");
            None
        }
    }
}

/// Initialize handshake returns the expected protocol version.
#[test]
fn initialize_returns_protocol_version() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    let response = runner.initialize().unwrap();
    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();

    assert_eq!(v["jsonrpc"], "2.0");
    assert!(v["id"].is_number());
    assert_eq!(v["result"]["protocolVersion"], "2024-11-05");
    assert!(v["result"]["capabilities"]["tools"].is_object());
    assert_eq!(v["result"]["serverInfo"]["name"], "mcp-context-server");
}

/// tools/list returns exactly 3 tools.
#[test]
fn tools_list_returns_three_tools() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    // Must initialize first
    runner.initialize().unwrap();

    let response = runner.list_tools().unwrap();
    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();

    assert_eq!(v["jsonrpc"], "2.0");
    let tools = v["result"]["tools"].as_array().expect("tools should be an array");
    assert_eq!(tools.len(), 3, "expected 3 tools, got {}", tools.len());

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"context.resolve"), "missing context.resolve");
    assert!(names.contains(&"context.list_caches"), "missing context.list_caches");
    assert!(names.contains(&"context.inspect_cache"), "missing context.inspect_cache");
}

/// tools/call for context.resolve returns a valid result.
#[test]
fn tools_call_resolve() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let response = runner
        .call_tool(
            "context.resolve",
            serde_json::json!({
                "cache": "minimal",
                "query": "hello",
                "budget": 4000
            }),
        )
        .unwrap();

    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
    assert_eq!(v["jsonrpc"], "2.0");

    let result = &v["result"];
    assert!(result["content"].is_array(), "result should have content array");

    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty(), "content should not be empty");
    assert_eq!(content[0]["type"], "text");

    // Parse the inner text as JSON to verify it's valid selection output
    let inner_text = content[0]["text"].as_str().unwrap();
    let inner: serde_json::Value = serde_json::from_str(inner_text.trim()).unwrap();
    assert!(inner["documents"].is_array());
    assert!(inner["selection"].is_object());
}

/// tools/call for context.list_caches returns cache entries.
#[test]
fn tools_call_list_caches() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let response = runner
        .call_tool("context.list_caches", serde_json::json!({}))
        .unwrap();

    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
    assert_eq!(v["jsonrpc"], "2.0");

    let content = v["result"]["content"].as_array().unwrap();
    assert!(!content.is_empty());

    let inner_text = content[0]["text"].as_str().unwrap();
    let inner: serde_json::Value = serde_json::from_str(inner_text.trim()).unwrap();
    assert!(inner["caches"].is_array());
}

/// tools/call for context.inspect_cache returns inspect data.
#[test]
fn tools_call_inspect_cache() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let response = runner
        .call_tool(
            "context.inspect_cache",
            serde_json::json!({ "cache": "minimal" }),
        )
        .unwrap();

    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
    assert_eq!(v["jsonrpc"], "2.0");

    let content = v["result"]["content"].as_array().unwrap();
    let inner_text = content[0]["text"].as_str().unwrap();
    let inner: serde_json::Value = serde_json::from_str(inner_text.trim()).unwrap();
    assert!(inner["cache_version"].is_string());
    assert!(inner["valid"].as_bool().unwrap());
}

/// Unknown method returns a method_not_found JSON-RPC error.
#[test]
fn unknown_method_returns_error() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let response = runner.send_unknown_method().unwrap();
    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();

    assert_eq!(v["jsonrpc"], "2.0");
    assert!(v["error"].is_object(), "should have error field");
    assert_eq!(v["error"]["code"], -32601, "should be method_not_found code");
}

/// Requesting a missing cache via MCP returns an error tool result.
#[test]
fn tools_call_missing_cache_error() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let response = runner
        .call_tool(
            "context.resolve",
            serde_json::json!({
                "cache": "nonexistent_cache_xyz",
                "query": "test",
                "budget": 100
            }),
        )
        .unwrap();

    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
    assert_eq!(v["jsonrpc"], "2.0");

    let result = &v["result"];
    assert_eq!(result["isError"], true, "should be an error result");

    let content = result["content"].as_array().unwrap();
    let inner_text = content[0]["text"].as_str().unwrap();
    let inner: serde_json::Value = serde_json::from_str(inner_text.trim()).unwrap();
    assert_eq!(inner["error"]["code"], "cache_missing");
}

/// Sequential stability: multiple identical requests produce identical responses.
/// Verifies no hidden state accumulation across calls.
#[test]
fn sequential_resolve_stability() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let mut responses = Vec::new();
    for _ in 0..3 {
        let response = runner
            .call_tool(
                "context.resolve",
                serde_json::json!({
                    "cache": "minimal",
                    "query": "hello",
                    "budget": 4000
                }),
            )
            .unwrap();

        let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
        // Extract the inner tool result text (ignoring JSON-RPC envelope which has different IDs)
        let inner_text = v["result"]["content"][0]["text"]
            .as_str()
            .expect("should have text content");
        responses.push(inner_text.to_string());
    }

    // All three inner responses must be identical
    assert_eq!(responses[0], responses[1], "response 1 != response 2");
    assert_eq!(responses[1], responses[2], "response 2 != response 3");
}

/// Sequential stability for inspect: multiple calls produce identical results.
#[test]
fn sequential_inspect_stability() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let mut responses = Vec::new();
    for _ in 0..3 {
        let response = runner
            .call_tool(
                "context.inspect_cache",
                serde_json::json!({ "cache": "minimal" }),
            )
            .unwrap();

        let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
        let inner_text = v["result"]["content"][0]["text"]
            .as_str()
            .expect("should have text content");
        responses.push(inner_text.to_string());
    }

    assert_eq!(responses[0], responses[1], "inspect response 1 != response 2");
    assert_eq!(responses[1], responses[2], "inspect response 2 != response 3");
}

/// MCP error response shape is frozen: exact JSON structure for cache_missing.
/// This locks the error contract for machine consumers.
#[test]
fn mcp_error_shape_frozen() {
    let cache_root = fixture::cache_path("minimal").parent().unwrap().to_path_buf();
    let mut runner = match mcp(&cache_root) {
        Some(r) => r,
        None => return,
    };

    runner.initialize().unwrap();

    let response = runner
        .call_tool(
            "context.resolve",
            serde_json::json!({
                "cache": "nonexistent_cache_xyz",
                "query": "test",
                "budget": 100
            }),
        )
        .unwrap();

    let v: serde_json::Value = serde_json::from_str(response.trim()).unwrap();
    let result = &v["result"];

    // Load frozen golden error shape
    let expected_str = fixture::expected("mcp_error_cache_missing");
    let expected: serde_json::Value = serde_json::from_str(&expected_str).unwrap();

    // Compare the tool result (not the JSON-RPC envelope, which has a variable ID)
    let actual: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(result).unwrap(),
    )
    .unwrap();

    assert_eq!(
        actual, expected,
        "MCP error response shape does not match frozen golden.\nActual:   {actual}\nExpected: {expected}"
    );

    // Additionally verify structural invariants:
    assert_eq!(result["isError"], true);
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1, "error should have exactly 1 content item");
    assert_eq!(content[0]["type"], "text");

    // Parse inner error and verify code + message
    let inner: serde_json::Value =
        serde_json::from_str(content[0]["text"].as_str().unwrap().trim()).unwrap();
    assert_eq!(inner["error"]["code"], "cache_missing");
    assert_eq!(inner["error"]["message"], "Cache does not exist");
}

/// All 6 MCP error codes are valid per the frozen schema.
#[test]
fn mcp_error_codes_frozen() {
    let schema = fixture::schema("mcp_error");

    // These are the 6 frozen error codes from the v0 contract
    let frozen_codes = [
        "cache_missing",
        "cache_invalid",
        "invalid_query",
        "invalid_budget",
        "io_error",
        "internal_error",
    ];

    for code in &frozen_codes {
        let error_json = serde_json::json!({
            "error": {
                "code": code,
                "message": format!("Test message for {code}")
            }
        });

        let validator = jsonschema::validator_for(&schema).unwrap();
        assert!(
            validator.is_valid(&error_json),
            "error code '{code}' should validate against mcp_error schema"
        );
    }

    // Verify that an unknown code does NOT validate
    let bad_code = serde_json::json!({
        "error": {
            "code": "unknown_error_code",
            "message": "This should not validate"
        }
    });

    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        !validator.is_valid(&bad_code),
        "unknown error code should NOT validate against mcp_error schema"
    );
}
