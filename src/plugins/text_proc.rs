use std::io::Write;
use std::process::{Command, Stdio};
use tokio::task::spawn_blocking;

use super::super::common::*;

// TODO: pass in via envs (main)
const PROC_UTIL: &str = "pandoc";
const PROC_ARGS: [&str; 4] = ["-f", "jira", "-t", "plain"];

// Helper function to run pandoc transformation
pub async fn transform_with_proc(content: String) -> ResBoxed<String> {
    // Spawn a tokio task for the synchronous process
    let result = spawn_blocking(move || {
        // Use pandoc to convert markdown to plain text
        let mut child = Command::new(PROC_UTIL)
            .args(PROC_ARGS)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", PROC_UTIL, e))?;

        // Write content to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write to util stdin: {}", e))?;
        }

        // Read the output
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for util output: {}", e))?;

        // Handle result
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| format!("Output is not valid UTF-8: {}", e))
        } else {
            Err(format!(
                "Util failed with exit code {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    })
    .await??;

    Ok(result)
}
