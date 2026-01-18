use crate::types::MarkdownChunk;
use anyhow::{Result, anyhow};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;

// Regex to fix italic formatting: `_word_` -> _word_
static ITALIC_BACKTICK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`(_[^_]+_)`").expect("Failed to compile ITALIC_BACKTICK_RE regex")
});
// Regex to fix reference link formatting: `[text]: url` -> [text]: url
static LINK_BACKTICK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`(\[[^\]]+\]:\s*[^\n]+)`").expect("Failed to compile LINK_BACKTICK_RE regex")
});

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Translator for interacting with Ollama
pub struct Translator {
    client: Client,
    model: String,
    base_url: String,
}

impl Translator {
    /// Create a new translator
    pub fn new(model: &str, base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            model: model.to_string(),
            base_url: base_url.to_string(),
        }
    }

    /// Build translation prompt with terminology
    fn build_prompt(&self, chunk: &MarkdownChunk, terminology_prompt: &str) -> String {
        format!(
            r#"You are a professional technical translator.

Translate the following English Rust documentation into natural Japanese.

{}

Rules:
- Do NOT translate code blocks (preserve ```language ... ``` blocks exactly)
- Use the provided terminology consistently
- Keep sentences concise and technical
- Preserve markdown formatting (headers, lists, links, etc.)
- IMPORTANT: Preserve italic formatting with underscores (_word_) exactly as in original
- IMPORTANT: For reference link sections at the end (format: [name]: url), translate ONLY the link text, do NOT add descriptions or explanations after the links

Text to translate:
{}"#,
            terminology_prompt, chunk.content
        )
    }

    /// Translate a single chunk
    pub async fn translate_chunk(
        &self,
        chunk: &MarkdownChunk,
        terminology_prompt: &str,
    ) -> Result<String> {
        let prompt = self.build_prompt(chunk, terminology_prompt);

        let req = OllamaRequest {
            model: &self.model,
            prompt: &prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        let res = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;

        let body: OllamaResponse = res.json().await?;

        // Apply post-processing to fix common formatting issues
        Ok(Self::post_process_translation(body.response))
    }

    /// Post-process translation to fix formatting issues
    fn post_process_translation(text: String) -> String {
        // Fix pattern where LLM added backticks around italic text: `_word_` -> `_word_
        let text = ITALIC_BACKTICK_RE.replace_all(&text, "$1").to_string();

        // Fix pattern where LLM added backticks around links: `[text]: url` -> `[text]: url`

        LINK_BACKTICK_RE.replace_all(&text, "$1").to_string()
    }

    /// Translate with retry logic
    pub async fn translate_with_retry(
        &self,
        chunk: &MarkdownChunk,
        terminology_prompt: &str,
        max_retries: usize,
    ) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.translate_chunk(chunk, terminology_prompt).await {
                Ok(result) => {
                    if attempt > 0 {
                        eprintln!("  ✓ Retry successful on attempt {}", attempt + 1);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("  ✗ Attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    // Exponential backoff
                    if attempt < max_retries - 1 {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(anyhow!(
            "Failed after {} retries: {:?}",
            max_retries,
            last_error
        ))
    }
}
