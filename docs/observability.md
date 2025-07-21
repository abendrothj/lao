# LAO Observability & Debugging

## Logging
- Each step logs lifecycle events: `init`, `pre_execute`, `execute`, `post_execute`, `shutdown` (with timestamps)
- Retry attempts and errors are logged per step
- Cache hits/misses/saves are logged
- All logs are tagged with step name and status

## Prompt Validation & Test Harness
- Use the CLI or test harness to validate prompt-to-workflow generation
- Structure-aware matcher compares generated and expected DAGs
- Logs diffs, highlights failures, suggests corrections

## Debugging Agentic Workflows
- Use `--dry-run` to simulate execution and check types
- Use `validate-prompts` to test prompt/LLM output
- Check logs for error traces, retry history, and plugin chain issues
- (Planned) Step-level debug and explainability features 