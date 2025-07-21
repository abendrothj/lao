# LAO Prompt Library

A collection of sample prompts and their expected YAML workflow outputs for testing, fine-tuning, and validation.

| Prompt | Generated Workflow |
|--------|--------------------|
| Summarize this Markdown doc and extract key ideas | workflow: "Markdown Summary"
steps:
  - run: MarkdownSummarizer
    input: "doc.md"
  - run: Tagger
    input_from: MarkdownSummarizer |
| Audit this config file for security risks | workflow: "Config Audit"
steps:
  - run: ConfigParser
    input: "config.yaml"
  - run: SecurityAuditor
    input_from: ConfigParser
  - run: Reporter
    input_from: SecurityAuditor |
| Refactor this Rust file and add comments | workflow: "Rust Refactor"
steps:
  - run: RustRefactor
    input: "main.rs"
  - run: CommentGenerator
    input_from: RustRefactor |
| Summarize this audio and create a todo list | workflow: "Audio Todo"
steps:
  - run: Whisper
    input: "meeting.wav"
  - run: Summarizer
    input_from: Whisper
  - run: TaskExtractor
    input_from: Summarizer | 