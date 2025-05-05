use super::super::common::*;

use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

// TODO: pass in via envs (main)
const PROC_UTIL: &str = "pandoc";
const PROC_ARGS: [&str; 4] = ["-f", "jira", "-t", "plain"];

// Helper function to run pandoc transformation
pub async fn transform(content: &str) -> ResBoxed<String> {
    // Spawn a tokio task for the synchronous process
    // Use pandoc to convert markdown to plain text
    let mut child = Command::new(PROC_UTIL)
        .args(PROC_ARGS)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", PROC_UTIL, e))?;

    // Write content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes()).await?
    } // auto closes when dropped

    // Read the output
    let output = child.wait_with_output().await?;

    // Handle result
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e).into())
    } else {
        Err(format!(
            "TEXT-PROC failed with exit code {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}
