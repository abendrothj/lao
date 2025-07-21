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

## Step Fields
- `run`: Plugin name
- `input`/`input_from`: Direct input or dependency
- `retry_count`, `retry_delay`: Retry config (ms)
- `cache_key`: Enable output caching
- `depends_on`: List of step names this step depends on

## Advanced Features
- **Retries**: Steps can retry on failure with exponential backoff.
- **Caching**: Steps with `cache_key` will skip execution if cached output exists.
- **Lifecycle Hooks**: Plugins can run custom logic before/after execution. 