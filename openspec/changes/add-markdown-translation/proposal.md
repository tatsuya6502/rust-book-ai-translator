# Change: Markdown Translation with Terminology Management

## Why

The Rust Book translation project needs a reliable way to translate English markdown documentation to Japanese using local LLMs (Ollama) while ensuring:
- Consistent technical terminology across translations
- Preservation of code blocks and markdown formatting
- Ability to handle large files through chunking
- Resume capability for long-running translations

## What Changes

- **NEW**: Markdown translation engine with Ollama integration
- **NEW**: Terminology glossary system (YAML-based) with per-chunk term extraction
- **NEW**: Smart chunking algorithm (~800 tokens) that respects code blocks and paragraph boundaries
- **NEW**: Progress checkpointing system (every 5 chunks) for resume capability
- **NEW**: Retry logic with exponential backoff (up to 3 retries)
- **NEW**: Post-processing to fix common LLM formatting issues (backticks around italics/links)

All changes are additive - no breaking changes to existing functionality.

## Impact

- **New capabilities**: markdown-translation
- **Affected code**:
  - `src/main.rs` - Orchestration and progress management
  - `src/translator.rs` - Ollama API client with terminology-aware prompts
  - `src/markdown.rs` - Chunking and code block detection
  - `src/terminology.rs` - YAML glossary loading and term extraction
  - `src/types.rs` - Core data structures
- **External dependencies**:
  - regex, serde_yaml, lazy_static, tiktoken-rs (added)
  - tokio features updated (added "fs")
