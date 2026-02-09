# context-compat

Standalone compatibility test harness for the Context platform. Tests the CLI and MCP server **externally** — no dependency on `context-core` or `mcp-context-server` crates.

## What it tests

| Test suite | Purpose |
|---|---|
| `determinism` | Same input twice → byte-identical output; build path independence |
| `golden_outputs` | Output matches committed expected/ files |
| `backward_compat` | Pre-built v0 caches, exit code contracts, IO failure boundaries, future version handling |
| `schema_validation` | All outputs validate against frozen JSON Schemas |
| `protocol_compat` | MCP server JSON-RPC responses, sequential stability |
| `cross_version` | Current binary vs previous binary regression detection |

## Prerequisites

Build the CLI and MCP server binaries first:

```bash
cargo build --release -p context-cli
cargo build --release -p mcp-context-server
```

## Running tests

```bash
cd context-compat

CONTEXT_CLI_BIN=../context-cli/target/release/context \
MCP_SERVER_BIN=../mcp-context-server/target/release/mcp-context-server \
cargo test
```

Or use the Makefile:

```bash
make test
```

Tests skip gracefully if the env vars are not set.

### Cross-version regression testing

To compare current against a previous binary:

```bash
CONTEXT_CLI_BIN=../context-cli/target/release/context \
CONTEXT_PREV_BIN=/path/to/previous/context \
cargo test --test cross_version
```

## Environment variables

| Variable | Purpose |
|---|---|
| `CONTEXT_CLI_BIN` | Path to the current `context` CLI binary |
| `MCP_SERVER_BIN` | Path to the current `mcp-context-server` binary |
| `CONTEXT_PREV_BIN` | Path to a previous release `context` binary (optional) |

## Adding new test cases

1. **New query**: Add a JSON file to `fixtures/v0/queries/` with `{"query": "...", "budget": N}`.
2. **New expected output**: Run the query, capture stdout, save to `fixtures/v0/expected/`.
3. **New document set**: Add `.md` files to a new directory under `fixtures/v0/documents/`.
4. **Rebuild fixtures**: `make fixtures` regenerates caches and expected outputs from current binaries.
5. **New contract version**: Create `fixtures/v1/` with its own documents, caches, queries, and expected outputs.

## File layout

```
context-compat/
├── src/                       # Runners (CLI, MCP) and fixture helpers
├── tests/                     # Integration tests (cargo test)
├── fixtures/
│   └── v0/                    # v0 contract fixtures
│       ├── documents/         # Source .md files
│       ├── caches/            # Pre-built caches (committed)
│       ├── queries/           # Query fixtures as JSON
│       └── expected/          # Golden expected outputs
└── schemas/                   # JSON Schemas for output validation
```

## CLI exit code contract

| Code | Condition |
|---|---|
| 0 | Success |
| 1 | Usage error (argument parsing) |
| 2 | Invalid query |
| 3 | Invalid budget |
| 4 | Cache missing |
| 5 | Cache invalid |
| 6 | I/O error |
| 7 | Internal error |

---

"Context Engine" is a trademark of Context Engine Contributors. The software is open source under the [Apache License 2.0](LICENSE). The trademark is not licensed for use by third parties to market competing products or services without prior written permission.
