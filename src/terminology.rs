use crate::types::{Principle, TranslationTable};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

// Regex to extract base term by removing parenthetical notes
static PARENS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\(.*?\)\s*").unwrap());

/// Load translation table from YAML file
pub fn load_translation_table(path: &str) -> Result<TranslationTable> {
    let content = std::fs::read_to_string(path)?;
    let table: TranslationTable = serde_saphyr::from_str(&content)?;
    Ok(table)
}

/// Extract terms from the glossary that appear in the chunk
pub fn extract_relevant_terms(
    chunk: &str,
    glossary: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut relevant = HashMap::new();
    let chunk_lower = chunk.to_lowercase();

    // Sort by term length (longest first) to match multi-word terms first
    let mut terms: Vec<_> = glossary.iter().collect();
    terms.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));

    // Remove notes and context from keys when matching
    for (term, translation) in terms {
        // Extract the base English term (remove parenthetical notes like "(lockの)")
        let base_term = extract_base_term(term);

        // Create word boundary pattern for case-insensitive matching
        let pattern = format!(r"(?i)\b{}\b", regex::escape(&base_term));
        if let Ok(re) = Regex::new(&pattern)
            && re.is_match(&chunk_lower)
        {
            relevant.insert(term.clone(), translation.clone());

            // Limit to prevent prompt explosion
            if relevant.len() >= 15 {
                break;
            }
        }
    }

    relevant
}

/// Extract the base term by removing parenthetical notes
/// e.g., "acquire (lockの)" -> "acquire"
fn extract_base_term(term: &str) -> String {
    // Remove content in parentheses and surrounding whitespace
    PARENS_RE.replace(term, "").to_string()
}

/// Build the terminology prompt section
pub fn build_terminology_prompt(
    relevant_terms: &HashMap<String, String>,
    principles: &[Principle],
) -> String {
    let mut prompt = String::from("Translation Principles:\n");

    for principle in principles {
        prompt.push_str(&format!("- {}\n", principle.text));
        for note in &principle.notes {
            prompt.push_str(&format!("  - {}\n", note));
        }
        if let Some(ref reference) = principle.reference {
            prompt.push_str(&format!("  (Reference: {})\n", reference));
        }
    }

    prompt.push_str("\nRelevant Terminology:\n");

    // Sort alphabetically for consistency
    let mut sorted_terms: Vec<_> = relevant_terms.iter().collect();
    sorted_terms.sort_by(|a, b| a.0.cmp(b.0));

    for (term, translation) in sorted_terms {
        prompt.push_str(&format!("- \"{}\" -> \"{}\"\n", term, translation));
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_term() {
        assert_eq!(extract_base_term("acquire (lockの)"), "acquire");
        assert_eq!(extract_base_term("simple term"), "simple term");
        assert_eq!(extract_base_term("term with (note)"), "term with");
    }
}
