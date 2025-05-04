use memchr::*;
use std::ops::Range;

#[derive(Debug)]
pub struct TagSegm {
    pub start_marker: Range<usize>,
    pub text: Range<usize>,
    pub end_marker: Range<usize>,
}

pub fn find_segm(chunk: &str, begin: &str, closing: &str) -> Option<Range<usize>> {
    // TODO: pass finder(s) as args since they might be costly to build
    let chunk_b = chunk.as_bytes();
    let begin_b = begin.as_bytes();
    let closing_b = closing.as_bytes();

    let begin_f = memmem::Finder::new(begin_b);
    let begin_pos = begin_f.find(chunk_b)?;

    let reset_pos = begin_pos + begin_b.len();
    let shifted_chunk = &chunk_b[reset_pos..];
    let closing_f = memmem::Finder::new(closing_b);
    let closing_pos = closing_f.find(shifted_chunk)?;

    Some(reset_pos..(reset_pos + closing_pos))
}

// TODO: this is not optimal, might pickup NON tags
pub fn parse_jira_markup(input: &str) -> Vec<TagSegm> {
    let mut results = Vec::new();
    let mut offset = 0;

    while offset < input.len() {
        let begin = "{{";
        let closing = "}}";
        if let Some(text) = find_segm(&input[offset..], begin, closing) {
            let abs_start = offset + text.start;
            let abs_end = offset + text.end;
            let new_offset = abs_end + closing.len();

            let start_marker = abs_start..(abs_start + begin.len());
            let end_marker = abs_end..new_offset;

            results.push(TagSegm {
                start_marker,
                text,
                end_marker,
            });

            offset = new_offset;
        } else {
            break;
        }
    }

    results
    // 2nd pass: if first and last char of text = {} don't mess further
    // 3rd pass: if tag == noformat KEEP all text
}

fn is_base64(char: char) -> bool {
    matches!(char,

        'A'..='Z' |
        'a'..='z' |
        '0'..='9' |
        '+' | '/' | '='
    )
}

// Extracts mime type and base64 data from image pattern
pub fn parse_base64_image(s: &str) -> Option<(&str, &str, &str)> {
    let s = s.strip_prefix("data:image/")?;
    let (mime_type, s) = s.split_once(';')?;
    let s = s.strip_prefix("base64,")?;
    let (base64_data, rest) = s.split_once(|c| !is_base64(c))?;
    Some((mime_type, base64_data, rest))
}
#[cfg(test)]
mod tests {
    use super::*;
    const FIX : &str = "Text  {color}data:image/png;base64,ABC123 {noformat} more {{data:image/jpeg;base64,XYZ456}}";

    #[test]
    fn remove_images() {
        let fs = parse_jira_markup(&FIX);

        let fst = fs.get(0).unwrap();
        assert_eq!("color", &FIX[fst.text.start..fst.text.end]);
        let snd = fs.get(1).unwrap();

        //assert_eq!("noformat", &FIX[snd.text.start..snd.text.end]);
        assert_eq!(3, fs.len());
    }
}
