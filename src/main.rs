mod markdown;
mod terminology;
mod translator;
mod types;

use anyhow::Result;
use std::path::Path;
use tokio::fs;
use translator::Translator;
use types::TranslationCheckpoint;

use markdown::chunk_markdown;
use terminology::{build_terminology_prompt, extract_relevant_terms, load_translation_table};

const INPUT_FILE: &str = "ch02-00-guessing-game-tutorial-en.md";
const OUTPUT_FILE: &str = "ch02-00-guessing-game-tutorial-ja.md";
const TRANSLATION_TABLE: &str = "translation-table.yaml";
const PROGRESS_FILE: &str = "translation_progress.json";
const TARGET_TOKENS: usize = 800;
const MODEL: &str = "translategemma:12b-it-q4_K_M";
const OLLAMA_URL: &str = "http://localhost:11434";

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting sequential markdown translation");
    println!("📖 Input: {}", INPUT_FILE);
    println!("📝 Output: {}", OUTPUT_FILE);
    println!("📚 Translation table: {}", TRANSLATION_TABLE);
    println!("🎯 Target chunk size: {} tokens", TARGET_TOKENS);
    println!();

    // Load translation table
    println!("⏳ Loading translation table...");
    let table = load_translation_table(TRANSLATION_TABLE)?;
    println!(
        "  ✓ Loaded {} principles and {} glossary entries",
        table.principles.len(),
        table.glossary.len()
    );
    println!();

    // Read input markdown
    println!("⏳ Reading input markdown...");
    let input = fs::read_to_string(INPUT_FILE).await?;
    let input_tokens = markdown::estimate_tokens(&input);
    println!(
        "  ✓ Input file size: {} bytes (~{} tokens)",
        input.len(),
        input_tokens
    );
    println!();

    // Chunk the markdown
    println!("⏳ Chunking markdown...");
    let chunks = chunk_markdown(&input, TARGET_TOKENS)?;
    println!("  ✓ Created {} chunks", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!(
            "    Chunk {}: {} tokens, code: {}",
            i + 1,
            chunk.token_count,
            chunk.contains_code
        );
    }
    println!();

    // Check for existing progress
    let (start_chunk, mut translated_chunks) = if Path::new(PROGRESS_FILE).exists() {
        println!("🔄 Found existing progress, resuming...");
        let progress_content = fs::read_to_string(PROGRESS_FILE).await?;
        let checkpoint: TranslationCheckpoint = serde_json::from_str(&progress_content)?;
        println!(
            "  ✓ Resuming from chunk {} of {}",
            checkpoint.completed_chunks, checkpoint.total_chunks
        );
        (checkpoint.completed_chunks, checkpoint.translated_content)
    } else {
        println!("🆕 Starting fresh translation");
        (0, Vec::new())
    };
    println!();

    // Initialize translator
    let translator = Translator::new(MODEL, OLLAMA_URL);

    // Translate chunks
    let total_chunks = chunks.len();
    let start_time = std::time::Instant::now();

    for (i, chunk) in chunks.iter().enumerate().skip(start_chunk) {
        let progress_percent = ((i + 1) as f64 / total_chunks as f64) * 100.0;
        println!(
            "🔄 Translating chunk {}/{} ({:.1}%) - {} tokens",
            i + 1,
            total_chunks,
            progress_percent,
            chunk.token_count
        );

        // Extract relevant terminology for this chunk
        let relevant_terms = extract_relevant_terms(&chunk.content, &table.glossary);
        let terminology_prompt = build_terminology_prompt(&relevant_terms, &table.principles);

        if !relevant_terms.is_empty() {
            println!(
                "  📖 Using {} relevant terminology terms",
                relevant_terms.len()
            );
        }

        // Translate with retry
        match translator
            .translate_with_retry(chunk, &terminology_prompt, 3)
            .await
        {
            Ok(translated) => {
                translated_chunks.push(translated);
                println!("  ✓ Translated successfully");
            }
            Err(e) => {
                eprintln!("  ✗ Failed to translate chunk: {}", e);
                eprintln!("  ⚠ Saving progress before exiting...");
                save_checkpoint(&translated_chunks, i, total_chunks).await?;
                return Err(e);
            }
        }

        // Save checkpoint after each chunk
        save_checkpoint(&translated_chunks, i + 1, total_chunks).await?;
        let elapsed = start_time.elapsed();
        let avg_time = elapsed / (i + 1) as u32;
        let remaining = avg_time * (total_chunks - i - 1) as u32;
        println!("  💾 Progress saved. ETA: {:?}", remaining);

        println!();
    }

    // Assemble and save output
    println!("⏳ Assembling translated chunks...");
    let output = translated_chunks.join("\n\n");
    let output_size = output.len();
    fs::write(OUTPUT_FILE, output).await?;
    println!("  ✓ Output written to {}", OUTPUT_FILE);
    println!();

    // Clean up progress file
    if Path::new(PROGRESS_FILE).exists() {
        fs::remove_file(PROGRESS_FILE).await?;
        println!("  ✓ Cleaned up progress file");
    }

    let elapsed = start_time.elapsed();
    println!("✅ Translation completed in {:?}", elapsed);
    println!("📊 Statistics:");
    println!("  - Total chunks: {}", total_chunks);
    println!("  - Input tokens: {}", input_tokens);
    println!("  - Output size: {} bytes", output_size);

    Ok(())
}

/// Save checkpoint for resume capability
async fn save_checkpoint(
    translated_chunks: &[String],
    completed: usize,
    total: usize,
) -> Result<()> {
    let checkpoint = TranslationCheckpoint {
        completed_chunks: completed,
        total_chunks: total,
        translated_content: translated_chunks.to_vec(),
        timestamp: std::time::SystemTime::now(),
    };

    let json = serde_json::to_string_pretty(&checkpoint)?;
    fs::write(PROGRESS_FILE, json).await?;
    Ok(())
}
