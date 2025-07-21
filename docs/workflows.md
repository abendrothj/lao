# LAO Workflows

## Workflow YAML Format
A workflow is a list of steps, each specifying a plugin to run, its inputs, and optional config.

```yaml
workflow: "Summarize Meeting"
steps:
  - run: Whisper
    input: "meeting.wav"
    retry_count: 3
    retry_delay: 1000
    cache_key: "whisper_meeting"
  - run: Summarizer
    input_from: Whisper
    cache_key: "summary_meeting"
  - run: Tagger
    input_from: Summarizer
```

## Prompt-Generated Workflows
- Use the CLI or UI to generate workflows from natural language prompts
- Example:
  ```bash
  lao prompt "Refactor this Rust file and add comments"
  ```
  Output:
  ```yaml
  workflow: "Rust Refactor"
  steps:
    - run: RustRefactor
      input: "main.rs"
    - run: CommentGenerator
      input_from: RustRefactor
  ```

## Advanced Features (Planned)
- **Conditional/Branching Steps**: if/else, loops, parameterized flows
- **Parameter Injection**: Securely pass secrets, user data, etc.
- **Multi-modal Input**: Files, voice, etc.

## Contributing Workflows
- Add new prompt/workflow pairs to the prompt library for validation and LLM tuning 