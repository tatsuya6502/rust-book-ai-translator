# Markdown Translation Capability Specification

## ADDED Requirements

### Requirement: Markdown Chunking

The system SHALL split markdown content into chunks of approximately 800 tokens while preserving content integrity.

#### Scenario: Successful chunking with code blocks
- **GIVEN** a markdown file containing code blocks and paragraphs
- **WHEN** the content is chunked with a target of 800 tokens
- **THEN** chunks are created with ~800 tokens each
- **AND** code blocks (```...```) are never split across chunks
- **AND** chunk boundaries respect paragraph breaks (double newlines)
- **AND** chunk boundaries respect sentence endings (. ! ?)

#### Scenario: Large paragraph handling
- **GIVEN** a single paragraph exceeding 800 tokens
- **WHEN** the content is chunked
- **THEN** the paragraph is split at sentence boundaries
- **AND** no split occurs mid-sentence if possible

#### Scenario: Code block preservation metadata
- **GIVEN** markdown content with code blocks
- **WHEN** chunks are created
- **THEN** each chunk includes metadata (contains_code: boolean)
- **AND** token count is accurately estimated using tiktoken

### Requirement: Terminology Management

The system SHALL load and apply a YAML glossary for consistent technical terminology.

#### Scenario: Glossary loading
- **GIVEN** a translation-table.yaml file with principles and glossary
- **WHEN** the system starts
- **THEN** the file is parsed successfully
- **AND** principles are extracted into memory
- **AND** glossary entries are loaded as a HashMap

#### Scenario: Relevant term extraction
- **GIVEN** a glossary with 200+ entries
- **AND** a chunk containing technical terms
- **WHEN** terms are extracted for the chunk
- **THEN** only terms present in the chunk are selected
- **AND** terms are matched case-insensitively with word boundaries
- **AND** multi-word terms are prioritized over single-word terms
- **AND** extraction is limited to 15 most relevant terms

#### Scenario: Terminology prompt construction
- **GIVEN** extracted relevant terms and translation principles
- **WHEN** the terminology prompt is built
- **THEN** principles are included at the top
- **AND** terms are formatted as "english -> japanese" mappings
- **AND** terms are sorted alphabetically for consistency

### Requirement: Ollama Translation

The system SHALL translate markdown chunks using the Ollama API with terminology-aware prompts.

#### Scenario: Successful translation with terminology
- **GIVEN** a markdown chunk and terminology prompt
- **AND** Ollama server running at http://localhost:11434
- **WHEN** the chunk is translated
- **THEN** the request is sent to POST /api/generate
- **AND** the model parameter is set to "translategemma:12b-it-q4_K_M"
- **AND** the terminology is included in the system prompt
- **AND** code blocks are preserved in the output
- **AND** the translation is returned as a string

#### Scenario: Translation retry on failure
- **GIVEN** a chunk to translate
- **AND** the Ollama request fails
- **WHEN** translation is attempted
- **THEN** the system retries up to 3 times
- **AND** retries use exponential backoff (100ms * 2^attempt)
- **AND** failure details are logged to stderr
- **AND** an error is returned if all retries fail

#### Scenario: Translation timeout
- **GIVEN** a slow Ollama response
- **WHEN** the request takes longer than 300 seconds
- **THEN** the request times out
- **AND** an error is returned
- **AND** retry logic is triggered

### Requirement: Post-Processing

The system SHALL apply post-processing to fix common LLM formatting issues.

#### Scenario: Italic formatting preservation
- **GIVEN** translated text with backticks around italics: `` `_word_` ``
- **WHEN** post-processing is applied
- **THEN** the extra backticks are removed
- **AND** the result is `_word_` (preserved as-is)

#### Scenario: Reference link formatting preservation
- **GIVEN** translated text with backticks around reference links: `[text]: url`
- **WHEN** post-processing is applied
- **THEN** the extra backticks are removed
- **AND** the link format is preserved

### Requirement: Progress Checkpointing

The system SHALL save translation progress for resume capability.

#### Scenario: Checkpoint creation
- **GIVEN** a translation in progress
- **AND** 5 chunks have been completed
- **WHEN** the checkpoint threshold is reached
- **THEN** a progress file is created (translation_progress.json)
- **AND** the file contains completed_chunks count
- **AND** the file contains total_chunks count
- **AND** the file contains all translated content strings
- **AND** the file contains a timestamp

#### Scenario: Resume from checkpoint
- **GIVEN** an existing progress file with 5 completed chunks
- **AND** a total of 14 chunks
- **WHEN** the system restarts
- **THEN** the system detects the progress file
- **AND** translation resumes from chunk 6
- **AND** completed chunks are not re-translated

#### Scenario: Checkpoint cleanup on completion
- **GIVEN** a translation that completes successfully
- **WHEN** all chunks are translated
- **THEN** the progress file is deleted
- **AND** a confirmation message is displayed

### Requirement: Translation Quality Rules

The system SHALL enforce specific rules in the translation prompt.

#### Scenario: Code block preservation
- **GIVEN** a prompt for translation
- **WHEN** the prompt is constructed
- **THEN** the prompt includes "Do NOT translate code blocks"
- **AND** the prompt specifies to preserve ```language ... ``` blocks exactly

#### Scenario: Italic formatting preservation
- **GIVEN** a prompt for translation
- **WHEN** the prompt is constructed
- **THEN** the prompt includes "Preserve italic formatting with underscores (_word_) exactly as in original"

#### Scenario: Reference link handling
- **GIVEN** a prompt for translation
- **WHEN** the prompt is constructed
- **THEN** the prompt includes "For reference link sections at the end, translate ONLY the link text, do NOT add descriptions or explanations"

### Requirement: Progress Reporting

The system SHALL provide real-time progress feedback during translation.

#### Scenario: Progress display
- **GIVEN** a translation in progress
- **WHEN** each chunk is processed
- **THEN** the current chunk number is displayed (e.g., "Translating chunk 5/14")
- **AND** the percentage complete is displayed
- **AND** the token count for the chunk is displayed
- **AND** the number of relevant terminology terms used is displayed

#### Scenario: ETA estimation
- **GIVEN** a translation with completed chunks
- **WHEN** progress is saved (every 5 chunks)
- **THEN** the average time per chunk is calculated
- **AND** the estimated time remaining is displayed

#### Scenario: Statistics reporting
- **GIVEN** a completed translation
- **WHEN** the translation finishes
- **THEN** total chunks processed is displayed
- **AND** input token count is displayed
- **AND** output file size is displayed
- **AND** total elapsed time is displayed
