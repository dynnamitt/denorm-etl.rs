use memchr::memmem;

fn process_content(input: String) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input.as_str();
    let mut image_counter = 0;

    // Jira patterns to remove
    let jira_patterns = &[b"{color}", b"{noformat}", b"{code}"];

    loop {
        // Find positions of next image or Jira pattern
        let next_image = memmem::find(remaining.as_bytes(), b"data:image/");
        let next_jira = jira_patterns
            .iter()
            .filter_map(|p| memmem::find(remaining.as_bytes(), p))
            .min();

        match (next_image, next_jira) {
            (Some(img_pos), Some(jira_pos)) if img_pos < jira_pos => {
                // Handle image first
                output.push_str(&remaining[..img_pos]);
                remaining = &remaining[img_pos..];
                if let Some((mime, _, rest)) = parse_base64_image(remaining) {
                    output.push_str(&format!("[IMAGE_{}_{}]", image_counter, mime));
                    image_counter += 1;
                    remaining = rest;
                } else {
                    output.push_str("data:image/");
                    remaining = &remaining["data:image/".len()..];
                }
            }
            (_, Some(jira_pos)) => {
                // Remove Jira markup
                output.push_str(&remaining[..jira_pos]);
                let pattern = jira_patterns
                    .iter()
                    .find(|p| memmem::find(&remaining.as_bytes()[jira_pos..], p) == Some(0))
                    .unwrap();
                remaining = &remaining[jira_pos + pattern.len()..];
            }
            (Some(img_pos), None) => {
                // Handle remaining image
                output.push_str(&remaining[..img_pos]);
                remaining = &remaining[img_pos..];
                if let Some((mime, _, rest)) = parse_base64_image(remaining) {
                    output.push_str(&format!("[IMAGE_{}_{}]", image_counter, mime));
                    image_counter += 1;
                    remaining = rest;
                } else {
                    output.push_str("data:image/");
                    remaining = &remaining["data:image/".len()..];
                }
            }
            (None, None) => {
                // Done
                output.push_str(remaining);
                break;
            }
        }
    }

    output
}

/// Extracts mime type and base64 data from image pattern
fn parse_base64_image(s: &str) -> Option<(&str, &str, &str)> {
    let s = s.strip_prefix("data:image/")?;
    let (mime_type, s) = s.split_once(';')?;
    let s = s.strip_prefix("base64,")?;
    let (base64_data, rest) = s.split_once(|c| c == '"' || c == '\'' || c == '<' || c == '>')?;
    Some((mime_type, base64_data, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_str() {
        let input = String::from(
        "Text {color}data:image/png;base64,ABC123{noformat} more {code}data:image/jpeg;base64,XYZ456",
    );
        let output = process_content(input);
        println!("{}", output);
    }
}
