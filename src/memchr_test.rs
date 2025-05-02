use memchr::memmem;

/// Extracts mime type and base64 data from image pattern
pub fn parse_base64_image(s: &str) -> Option<(&str, &str, &str)> {
    let s = s.strip_prefix("data:image/")?;
    let (mime_type, s) = s.split_once(';')?;
    let s = s.strip_prefix("base64,")?;
    let (base64_data, rest) = s.split_once(|c| !is_base64(c))?;
    Some((mime_type, base64_data, rest))
}

fn is_base64(char: char) -> bool {
    matches!(char,

        'A'..='Z' |
        'a'..='z' |
        '0'..='9' |
        '+' | '/' | '='
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIX : &str = "Text {color}data:image/png;base64,ABC123{noformat} more {code}data:image/jpeg;base64,XYZ456";

    #[test]
    fn remove_images() {
        let mut result = String::new();
        let mut current = FIX;

        while let Some((_mime, _chunk, rest)) = parse_base64_image(current) {
            result.push_str(rest);
            current = rest;
        }

        let expected = "Text data:image/png;base64 more data:image/jpeg;base64";
        assert_eq!(result, expected);
    }
}
