use std::io::Write;
use std::process::{Command, Stdio};
use tokio::task::spawn_blocking;

use super::super::common::*;
use regex::Regex;

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

#[allow(dead_code)]
// TODO: remove/drop since it probably is to slow for HUGE base64 strings
fn strip_base64_images(content: &str, ticket_key: &str) -> String {
    // Create output directory if it doesn't exist
    //fs::create_dir_all(output_dir)?; TAKEN OUT .. for now

    // Regex to find base64 encoded images in the content
    let re = Regex::new(r"data:image/([a-zA-Z0-9]+);base64,([A-Za-z0-9+/=]+)").unwrap();

    // NOTE: I dont like this for HUGE strings
    let mut result = content.to_string();
    let mut image_counter = 0;

    for cap in re.captures_iter(content) {
        let img_type = &cap[1];
        let _base64_data = &cap[2]; // skip scan/trace for now !!!!

        // Decode base64 data
        //let image_data = decode(base64_data)?;

        // Create a file for the image
        image_counter += 1;
        let file_name = format!("{}_{}.{}", ticket_key, image_counter, img_type);
        //let file_path = output_dir.join(&file_name);

        // Write image data to file
        //let mut file = fs::File::create(&file_path)?;
        //file.write_all(&image_data)?;

        // Replace the base64 data with a link to the image file
        let image_reference = format!("!{}!", file_name);
        println!("  Took out an image: {}", file_name);
        result = result.replace(&cap[0], &image_reference);
    }
    result
}
