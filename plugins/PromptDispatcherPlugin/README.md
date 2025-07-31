# PromptDispatcherPlugin

A plugin that uses a local LLM and a system prompt to generate executable workflows from natural language prompts.

## Input
- `text` (string): The user prompt describing the desired workflow.

## Output
- (string): YAML workflow specification.

## Example Usage
```sh
lao prompt "Summarize this Markdown doc and extract key ideas"
```

## Example Output
```yaml
workflow: "Markdown Summary"
steps:
  - run: MarkdownSummarizer
    input: "doc.md"
  - run: Tagger
    input_from: MarkdownSummarizer
```

## Usage
Use via the CLI or UI to generate workflows from prompts. See [Architecture](../../docs/architecture.md) for more details. 