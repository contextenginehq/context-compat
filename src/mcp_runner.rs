use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// Runner that spawns an MCP server process and communicates via JSON-RPC over stdin/stdout.
pub struct McpRunner {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
    next_id: AtomicU64,
}

impl McpRunner {
    /// Spawn a new MCP server process.
    ///
    /// - `bin`: Path to the `mcp-context-server` binary.
    /// - `cache_root`: Directory to set as `CONTEXT_CACHE_ROOT`.
    pub fn new(bin: impl Into<PathBuf>, cache_root: &Path) -> Result<Self, std::io::Error> {
        let bin = bin.into();
        let mut child = Command::new(&bin)
            .env("CONTEXT_CACHE_ROOT", cache_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout was piped");
        let reader = BufReader::new(stdout);

        Ok(Self {
            child,
            reader,
            next_id: AtomicU64::new(1),
        })
    }

    /// Create an MCP runner from the `MCP_SERVER_BIN` environment variable.
    /// Returns `None` if the variable is not set.
    pub fn from_env(cache_root: &Path) -> Option<Result<Self, std::io::Error>> {
        std::env::var("MCP_SERVER_BIN")
            .ok()
            .map(|p| Self::new(p, cache_root))
    }

    /// Send a raw JSON-RPC request string and read one response line.
    pub fn send(&mut self, request_json: &str) -> Result<String, std::io::Error> {
        let stdin = self.child.stdin.as_mut().expect("stdin was piped");
        writeln!(stdin, "{}", request_json)?;
        stdin.flush()?;

        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line)
    }

    /// Send the `initialize` JSON-RPC handshake.
    pub fn initialize(&mut self) -> Result<String, std::io::Error> {
        let id = self.next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "context-compat-test",
                    "version": "0.1.0"
                }
            }
        });
        self.send(&request.to_string())
    }

    /// Send `tools/list` and return the response.
    pub fn list_tools(&mut self) -> Result<String, std::io::Error> {
        let id = self.next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/list",
            "params": {}
        });
        self.send(&request.to_string())
    }

    /// Send `tools/call` for a specific tool with arguments.
    pub fn call_tool(
        &mut self,
        name: &str,
        arguments: Value,
    ) -> Result<String, std::io::Error> {
        let id = self.next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });
        self.send(&request.to_string())
    }

    /// Send a request with an unknown method to test error handling.
    pub fn send_unknown_method(&mut self) -> Result<String, std::io::Error> {
        let id = self.next_id();
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "unknown/method",
            "params": {}
        });
        self.send(&request.to_string())
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Drop for McpRunner {
    fn drop(&mut self) {
        // Close stdin to signal the server to shut down, then wait.
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}
