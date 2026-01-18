# Project Context

## Purpose
A Rust-based AI translation tool that translates English Rust documentation (such as "The Rust Programming Language" book) into Japanese using local Large Language Models (LLMs) via Ollama.

The tool:
- Chunks markdown content into manageable pieces (~800 tokens)
- Translates each chunk while preserving code blocks and formatting
- Uses a terminology glossary for consistent translations
- Supports progress checkpointing and resume capability
- Runs translations sequentially with retry logic

## Tech Stack
- **Rust 2024 Edition** - Primary language
- **Tokio** - Async runtime (multi-threaded scheduler, filesystem support)
- **Reqwest** - HTTP client (JSON support, rustls TLS)
- **Regex** - Pattern matching and text processing
- **Tiktoken-rs** - Token counting for chunking

## Project Conventions

### Code Style
- Error handling via `anyhow::Result` with `?` operator
- Serde derives for data structures (`Debug`, `Clone`, `Serialize`, `Deserialize`)
- Async/await patterns with tokio runtime
- Console output uses Japanese text with emoji indicators (🚀, ⏳, ✓, ✗, etc.)
- Consts defined in SCREAMING_SNAKE_CASE for configuration values

### Architecture Patterns
- **Modular design** with separate modules:
  - `types` - Core data structures
  - `markdown` - Markdown chunking and parsing
  - `terminology` - Glossary and translation principles
  - `translator` - Ollama API interaction
  - `main` - Orchestration and progress management
- **Chunk-based processing** - Content split by token count, preserving code blocks
- **Checkpoint/resume** - JSON progress file saves state every 1 chunk
- **Retry logic** - Failed translations retry up to 3 times before failing

### Testing Strategy
- No test suite currently implemented
- Manual testing through actual translation runs

### Git Workflow
- Main branch: `main`
- No specific commit conventions documented

## Domain Context

### Translation Principles
The project follows Japanese technical documentation conventions:
- Use katakana terms where natural (avoid forced Japanese translations)
- Remove long vowel marks (ー) at the end of 3+ syllable katakana words (JIS Z 8301:2011 standard)
- Keep syntax keywords in English (e.g., `match`, `if`, `let`)
- Preserve technical terminology consistency via glossary

### Content Handling
- **Code blocks** - Must NOT be translated, preserved exactly as-is
- **Markdown formatting** - Headers, lists, links, italics preserved
- **Reference links** - Only translate link text, not add descriptions

## Important Constraints
- **Token limit** - Chunks target ~800 tokens for model processing
- **Timeout** - 300 second timeout for Ollama API requests
- **Local LLM** - Requires Ollama server running locally
- **File format** - Expects markdown input, produces markdown output

## External Dependencies

### Ollama API
- **Local server**: http://localhost:11434
- **Model**: translategemma:12b-it-q4_K_M
- **Endpoint**: POST /api/generate
- **Required for**: All translation operations

### Configuration Files
- `translation-table.yaml` - Glossary and translation principles
- Input/output files defined as consts in main.rs
