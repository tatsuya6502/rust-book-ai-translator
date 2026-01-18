use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

/// Checkpoint for resume capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationCheckpoint {
    pub completed_chunks: usize,
    pub total_chunks: usize,
    pub translated_content: Vec<String>,
    pub timestamp: std::time::SystemTime,
}

impl TranslationCheckpoint {
    /// Validate that the checkpoint matches the current chunking configuration
    pub fn validate(&self, current_total_chunks: usize) -> Result<()> {
        let validation_errors = [
            (
                self.total_chunks == current_total_chunks,
                format!(
                    "Checkpoint total_chunks ({}) does not match current chunks ({})",
                    self.total_chunks, current_total_chunks
                ),
            ),
            (
                self.completed_chunks <= self.total_chunks,
                format!(
                    "Checkpoint completed_chunks ({}) exceeds total_chunks ({})",
                    self.completed_chunks, self.total_chunks
                ),
            ),
            (
                self.translated_content.len() == self.completed_chunks,
                format!(
                    "Checkpoint translated_content length ({}) does not match completed_chunks ({})",
                    self.translated_content.len(),
                    self.completed_chunks
                ),
            ),
        ];

        let invalid: Vec<&str> = validation_errors
            .iter()
            .filter(|(valid, _)| !*valid)
            .map(|(_, msg)| msg.as_str())
            .collect();

        if !invalid.is_empty() {
            anyhow::bail!("Checkpoint validation failed:\n{}", invalid.join("\n"));
        }

        Ok(())
    }
}

/// Load checkpoint from progress file with validation
pub async fn load_checkpoint(
    progress_file: &str,
    current_total_chunks: usize,
) -> Result<Option<TranslationCheckpoint>> {
    if !Path::new(progress_file).exists() {
        return Ok(None);
    }

    let progress_content = fs::read_to_string(progress_file)
        .await
        .context("Failed to read progress file")?;

    let checkpoint: TranslationCheckpoint =
        serde_json::from_str(&progress_content).context("Failed to parse checkpoint file")?;

    checkpoint.validate(current_total_chunks)?;

    Ok(Some(checkpoint))
}

/// Save checkpoint for resume capability
pub async fn save_checkpoint(
    translated_chunks: &[String],
    completed: usize,
    total: usize,
    progress_file: &str,
) -> Result<()> {
    let checkpoint = TranslationCheckpoint {
        completed_chunks: completed,
        total_chunks: total,
        translated_content: translated_chunks.to_vec(),
        timestamp: std::time::SystemTime::now(),
    };

    let json = serde_json::to_string_pretty(&checkpoint)?;
    fs::write(progress_file, json).await?;
    Ok(())
}
