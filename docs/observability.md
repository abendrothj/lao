# LAO Observability & Debugging

## Logging
- Each step logs lifecycle events: `init`, `pre_execute`, `execute`, `post_execute`, `shutdown` (with timestamps)
- Retry attempts and errors are logged per step
- Cache hits/misses/saves are logged
- All logs are tagged with step name and status

## Step Status
- `pending`: Not started
- `completed`: Ran successfully
- `failed`: Error after all retries
- `cache`: Output loaded from cache

## Debugging Tips
- Use `--dry-run` to simulate execution and check types
- Check logs for error traces and retry history
- Use `validate` to check plugin availability and workflow structure 