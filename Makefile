.PHONY: build test fixtures clean

CONTEXT_CLI_BIN ?= ../context-cli/target/release/context
MCP_SERVER_BIN  ?= ../mcp-context-server/target/release/mcp-context-server

build:
	cargo build

test:
	CONTEXT_CLI_BIN=$(CONTEXT_CLI_BIN) \
	MCP_SERVER_BIN=$(MCP_SERVER_BIN) \
	cargo test -- --nocapture

# Rebuild pre-built v0 caches and expected outputs from current binaries.
# Does NOT rebuild the future_version fixture (hand-crafted).
fixtures: fixtures-caches fixtures-expected

fixtures-caches:
	rm -rf fixtures/v0/caches/minimal fixtures/v0/caches/realistic fixtures/v0/caches/tie_break
	$(CONTEXT_CLI_BIN) build \
		--sources fixtures/v0/documents/minimal \
		--cache fixtures/v0/caches/minimal
	$(CONTEXT_CLI_BIN) build \
		--sources fixtures/v0/documents/realistic \
		--cache fixtures/v0/caches/realistic
	$(CONTEXT_CLI_BIN) build \
		--sources fixtures/v0/documents/tie_break \
		--cache fixtures/v0/caches/tie_break

fixtures-expected:
	$(CONTEXT_CLI_BIN) resolve \
		--cache fixtures/v0/caches/minimal \
		--query deployment --budget 4000 \
		> fixtures/v0/expected/minimal_basic.json
	$(CONTEXT_CLI_BIN) resolve \
		--cache fixtures/v0/caches/minimal \
		--query deployment --budget 0 \
		> fixtures/v0/expected/minimal_zero_budget.json
	$(CONTEXT_CLI_BIN) resolve \
		--cache fixtures/v0/caches/realistic \
		--query deployment --budget 4000 \
		> fixtures/v0/expected/realistic_basic.json
	$(CONTEXT_CLI_BIN) resolve \
		--cache fixtures/v0/caches/realistic \
		--query "deployment security" --budget 4000 \
		> fixtures/v0/expected/realistic_multi_term.json
	$(CONTEXT_CLI_BIN) inspect \
		--cache fixtures/v0/caches/minimal \
		> fixtures/v0/expected/inspect_minimal.json
	$(CONTEXT_CLI_BIN) inspect \
		--cache fixtures/v0/caches/realistic \
		> fixtures/v0/expected/inspect_realistic.json
	$(CONTEXT_CLI_BIN) resolve \
		--cache fixtures/v0/caches/tie_break \
		--query deployment --budget 4000 \
		> fixtures/v0/expected/tie_break_ordering.json
	$(CONTEXT_CLI_BIN) resolve \
		--cache fixtures/v0/caches/tie_break \
		--query xyznotfound --budget 4000 \
		> fixtures/v0/expected/tie_break_zero_score.json

clean:
	cargo clean
