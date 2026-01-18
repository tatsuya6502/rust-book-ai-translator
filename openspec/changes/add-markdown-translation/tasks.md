# Implementation Tasks

## 1. Project Setup
- [x] 1.1 Update Cargo.toml with new dependencies (regex, serde_yaml, lazy_static, tiktoken-rs)
- [x] 1.2 Update tokio features to include "fs"
- [x] 1.3 Rename package from "hello-gemma" to "rust-book-translator"

## 2. Core Data Structures
- [x] 2.1 Create `src/types.rs` with TranslationTable, Principle, MarkdownChunk, TranslationCheckpoint
- [x] 2.2 Add Serde derives for serialization (Debug, Clone, Serialize, Deserialize)

## 3. Terminology Management
- [x] 3.1 Create `src/terminology.rs` module
- [x] 3.2 Implement `load_translation_table()` to parse YAML glossary
- [x] 3.3 Implement `extract_relevant_terms()` to find terms in each chunk
- [x] 3.4 Implement `extract_base_term()` helper to remove parenthetical notes
- [x] 3.5 Implement `build_terminology_prompt()` to construct prompt sections
- [x] 3.6 Add unit tests for base term extraction

## 4. Markdown Processing
- [x] 4.1 Create `src/markdown.rs` module
- [x] 4.2 Implement `identify_code_blocks()` to detect ```...``` blocks
- [x] 4.3 Implement `estimate_tokens()` using tiktoken-rs
- [x] 4.4 Implement `find_break_point()` for safe chunk boundaries
- [x] 4.5 Implement `chunk_markdown()` with smart boundary detection
- [x] 4.6 Implement `split_large_paragraph()` for oversized paragraphs
- [x] 4.7 Add unit tests for code block detection and token estimation

## 5. Translation Engine
- [x] 5.1 Create `src/translator.rs` module
- [x] 5.2 Implement `Translator` struct with Ollama client
- [x] 5.3 Implement `build_prompt()` with terminology integration
- [x] 5.4 Implement `translate_chunk()` for single chunk translation
- [x] 5.5 Implement `translate_with_retry()` with exponential backoff
- [x] 5.6 Implement `post_process_translation()` to fix formatting issues
- [x] 5.7 Add 300-second timeout for API requests

## 6. Orchestration
- [x] 6.1 Update `src/main.rs` with modular imports
- [x] 6.2 Implement progress checkpointing (save every 5 chunks)
- [x] 6.3 Implement resume capability from checkpoint
- [x] 6.4 Add progress reporting with emoji indicators
- [x] 6.5 Implement cleanup of progress file on completion
- [x] 6.6 Add statistics reporting (chunks, tokens, file sizes)

## 7. Translation Quality Improvements
- [x] 7.1 Update prompt to preserve italic formatting (_word_)
- [x] 7.2 Update prompt to prevent adding descriptions after reference links
- [x] 7.3 Add post-processing regex to remove extra backticks around italics
- [x] 7.4 Add post-processing regex to remove extra backticks around links

## 8. Testing
- [x] 8.1 Create test-input.md for basic functionality testing
- [x] 8.2 Create test-fixes.md for formatting verification
- [x] 8.3 Run full translation on ch02-00-guessing-game-tutorial-en.md
- [x] 8.4 Verify code blocks are preserved
- [x] 8.5 Verify terminology is applied consistently
- [x] 8.6 Verify italic formatting and links are preserved
