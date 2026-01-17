use crate::types::MarkdownChunk;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::LazyLock;
use tiktoken_rs::tiktoken::cl100k_base;

// Code block pattern: ```language ... ```
static CODE_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?ms)^(```[^\n]*\n.*?```)").unwrap()
});
// Paragraph break: double newline
static PARAGRAPH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\n\n+").unwrap()
});
// Header pattern: ## Header
static HEADER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^#{1,6}\s").unwrap()
});
// Sentence ending: . ! ? followed by space
static SENTENCE_END_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[.!?]\s+").unwrap()
});
// Clause boundary: , or ;
static CLAUSE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[,;]\s+").unwrap()
});

/// Identify all code blocks and their positions
/// Returns Vec of (start, end, language)
pub fn identify_code_blocks(text: &str) -> Vec<(usize, usize, String)> {
    let mut blocks = Vec::new();
    for mat in CODE_BLOCK_RE.find_iter(text) {
        let content = &text[mat.start()..mat.end()];
        // Extract language if present
        let language = if let Some(lang_start) = content.strip_prefix("```") {
            if let Some(newline_pos) = lang_start.find('\n') {
                lang_start[..newline_pos].to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        blocks.push((mat.start(), mat.end(), language));
    }
    blocks
}

/// Check if a position is inside any code block
fn is_inside_code_block(pos: usize, code_blocks: &[(usize, usize, String)]) -> bool {
    for (start, end, _) in code_blocks {
        if pos >= *start && pos < *end {
            return true;
        }
    }
    false
}

/// Estimate token count using tiktoken
pub fn estimate_tokens(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    bpe.encode_with_special_tokens(text).len()
}

/// Find a safe break point in text that doesn't break code blocks
/// Priority: paragraph > header > sentence > clause > word boundary
fn find_break_point(
    text: &str,
    start_pos: usize,
    target_end: usize,
    code_blocks: &[(usize, usize, String)],
) -> Option<usize> {
    let search_text = &text[start_pos..target_end.min(text.len())];

    // First, check if target_end would split a code block
    if is_inside_code_block(target_end, code_blocks) {
        // If we're in a code block, extend to end of code block
        for (start, end, _) in code_blocks {
            if target_end >= *start && target_end < *end {
                return Some(*end);
            }
        }
    }

    // Search backwards from target_end for safe break points
    let mut last_para = None;
    let mut last_header = None;
    let mut last_sentence = None;
    let mut last_clause = None;
    let mut last_space = None;

    for (idx, _) in search_text.match_indices('\n').rev() {
        let abs_pos = start_pos + idx;

        // Skip if inside code block
        if is_inside_code_block(abs_pos, code_blocks) {
            continue;
        }

        // Check for paragraph break (double newline)
        if abs_pos > 0 && text.as_bytes()[abs_pos - 1] == b'\n' {
            last_para = Some(abs_pos + 1);
        }

        // Check for header
        if let Some(next_char) = text[abs_pos + 1..].chars().next() {
            if next_char == '#' {
                last_header = Some(abs_pos);
            }
        }

        // Check for sentence ending
        if abs_pos > 0 {
            let prev_char = text.as_bytes()[abs_pos - 1];
            if prev_char == b'.' || prev_char == b'!' || prev_char == b'?' {
                last_sentence = Some(abs_pos + 1);
            }
        }
    }

    // Search for clause and word boundaries
    for (idx, ch) in search_text.char_indices().rev() {
        let abs_pos = start_pos + idx;

        if is_inside_code_block(abs_pos, code_blocks) {
            continue;
        }

        if ch == ',' || ch == ';' {
            last_clause = Some(abs_pos + ch.len_utf8());
        } else if ch.is_whitespace() {
            last_space = Some(abs_pos + ch.len_utf8());
        }
    }

    // Return best break point (in priority order)
    last_para
        .or(last_header)
        .or(last_sentence)
        .or(last_clause)
        .or(last_space)
}

/// Chunk markdown content into segments of approximately target_tokens
/// Preserves code blocks and respects paragraph/sentence boundaries
pub fn chunk_markdown(content: &str, target_tokens: usize) -> Result<Vec<MarkdownChunk>> {
    let mut chunks = Vec::new();
    let code_blocks = identify_code_blocks(content);

    let mut current_start = 0;
    let mut current_text = String::new();

    // Split into paragraphs first
    let paragraphs: Vec<&str> = PARAGRAPH_RE.split(content).collect();

    for para in paragraphs {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }

        let potential_content = if current_text.is_empty() {
            para.to_string()
        } else {
            format!("{}\n\n{}", current_text, para)
        };

        let potential_tokens = estimate_tokens(&potential_content);

        // Check if this paragraph would exceed the limit
        if potential_tokens <= target_tokens {
            current_text = potential_content;
        } else {
            // Would exceed limit - need to split

            // If current text is not empty, save it as a chunk
            if !current_text.is_empty() {
                let token_count = estimate_tokens(&current_text);
                let contains_code = CODE_BLOCK_RE.is_match(&current_text);
                chunks.push(MarkdownChunk::new(
                    current_text.clone(),
                    current_start,
                    current_start + current_text.len(),
                    contains_code,
                    token_count,
                ));
                current_start += current_text.len();
                current_text = String::new();
            }

            // Check if the paragraph itself is too large
            let para_tokens = estimate_tokens(para);
            if para_tokens > target_tokens {
                // Need to split the paragraph
                let split_chunks = split_large_paragraph(para, target_tokens, &code_blocks)?;
                for split_content in split_chunks.iter() {
                    let contains_code = CODE_BLOCK_RE.is_match(split_content);
                    let token_count = estimate_tokens(split_content);
                    chunks.push(MarkdownChunk::new(
                        split_content.clone(),
                        current_start,
                        current_start + split_content.len(),
                        contains_code,
                        token_count,
                    ));
                    current_start += split_content.len();
                }
            } else {
                // Paragraph fits, just start new chunk
                current_text = para.to_string();
            }
        }
    }

    // Don't forget the last chunk
    if !current_text.is_empty() {
        let token_count = estimate_tokens(&current_text);
        let contains_code = CODE_BLOCK_RE.is_match(&current_text);
        chunks.push(MarkdownChunk::new(
            current_text.clone(),
            current_start,
            current_start + current_text.len(),
            contains_code,
            token_count,
        ));
    }

    if chunks.is_empty() {
        return Err(anyhow!("No chunks were created from content"));
    }

    Ok(chunks)
}

/// Split a large paragraph into smaller chunks
fn split_large_paragraph(
    para: &str,
    target_tokens: usize,
    code_blocks: &[(usize, usize, String)],
) -> Result<Vec<String>> {
    let mut chunks = Vec::new();
    let mut current_start = 0;

    while current_start < para.len() {
        let remaining = &para[current_start..];
        let remaining_tokens = estimate_tokens(remaining);

        if remaining_tokens <= target_tokens {
            chunks.push(remaining.to_string());
            break;
        }

        // Find break point
        let target_end = current_start + (target_tokens * 3 / 4); // Use 75% to account for estimation errors
        if let Some(break_point) = find_break_point(para, current_start, target_end, code_blocks) {
            chunks.push(para[current_start..break_point].to_string());
            current_start = break_point;
        } else {
            // Force break at character position
            let break_pos = current_start + target_tokens.min(remaining.len());
            chunks.push(para[current_start..break_pos].to_string());
            current_start = break_pos;
        }
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_code_blocks() {
        let text = r#"
Some text

```rust
let x = 5;
```

More text
"#;
        let blocks = identify_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].2, "rust");
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "This is a simple test.";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_chunk_markdown() {
        let content = r#"
First paragraph.

Second paragraph.

```rust
let x = 5;
```

Third paragraph.
"#;
        let chunks = chunk_markdown(content, 100).unwrap();
        assert!(!chunks.is_empty());
    }
}
