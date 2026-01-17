use serde::{Deserialize, Serialize};

/// Translation table loaded from translation-table.yaml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationTable {
    pub principles: Vec<Principle>,
    pub glossary: std::collections::HashMap<String, String>,
}

/// A translation principle
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Principle {
    pub text: String,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    #[serde(rename = "reference")]
    pub reference: Option<String>,
}

/// A chunk of markdown content with metadata
#[derive(Debug, Clone)]
pub struct MarkdownChunk {
    pub content: String,
    pub contains_code: bool,
    pub token_count: usize,
}

impl MarkdownChunk {
    pub fn new(content: String, contains_code: bool, token_count: usize) -> Self {
        Self {
            content,
            contains_code,
            token_count,
        }
    }
}

/// Checkpoint for resume capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationCheckpoint {
    pub completed_chunks: usize,
    pub total_chunks: usize,
    pub translated_content: Vec<String>,
    pub timestamp: std::time::SystemTime,
}
