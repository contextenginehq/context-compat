use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Runner that invokes the `context` CLI binary via `std::process::Command`.
///
/// Binary path is resolved from an explicit path or the `CONTEXT_CLI_BIN` env var.
pub struct CliRunner {
    bin: PathBuf,
}

/// Result of a CLI invocation containing stdout, stderr, and exit code.
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CliOutput {
    fn from_output(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}

impl CliRunner {
    /// Create a runner from an explicit binary path.
    pub fn new(bin: impl Into<PathBuf>) -> Self {
        Self { bin: bin.into() }
    }

    /// Create a runner from the `CONTEXT_CLI_BIN` environment variable.
    /// Returns `None` if the variable is not set.
    pub fn from_env() -> Option<Self> {
        std::env::var("CONTEXT_CLI_BIN")
            .ok()
            .map(|p| Self::new(p))
    }

    /// Run `context build --sources <sources> --cache <cache> [--force]`.
    pub fn build(
        &self,
        sources: &Path,
        cache: &Path,
        force: bool,
    ) -> Result<CliOutput, std::io::Error> {
        let mut cmd = Command::new(&self.bin);
        cmd.arg("build")
            .arg("--sources")
            .arg(sources)
            .arg("--cache")
            .arg(cache);
        if force {
            cmd.arg("--force");
        }
        cmd.output().map(CliOutput::from_output)
    }

    /// Run `context resolve --cache <cache> --query <query> --budget <budget>`.
    /// Returns the raw CLI output.
    pub fn resolve(
        &self,
        cache: &Path,
        query: &str,
        budget: usize,
    ) -> Result<CliOutput, std::io::Error> {
        Command::new(&self.bin)
            .arg("resolve")
            .arg("--cache")
            .arg(cache)
            .arg("--query")
            .arg(query)
            .arg("--budget")
            .arg(budget.to_string())
            .output()
            .map(CliOutput::from_output)
    }

    /// Run `context inspect --cache <cache>`.
    /// Returns the raw CLI output.
    pub fn inspect(&self, cache: &Path) -> Result<CliOutput, std::io::Error> {
        Command::new(&self.bin)
            .arg("inspect")
            .arg("--cache")
            .arg(cache)
            .output()
            .map(CliOutput::from_output)
    }
}
