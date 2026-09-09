# LAO — Local AI Workflow Orchestrator

![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)

LAO is a headless engine and CLI for chaining local plugins into DAG-based
workflows. Plugins are dynamically loaded shared libraries built against a small,
stable C ABI; workflows are plain YAML. Everything runs locally and offline.

## What it does

- Loads plugins at runtime by `dlopen`-ing shared libraries (`.so`/`.dylib`/`.dll`)
  that export a known C vtable.
- Parses a YAML workflow into a DAG, validates it (plugin availability + I/O type
  compatibility), topologically sorts it, and executes it.
- Supports parallel execution by dependency level, per-step retries with backoff,
  on-disk output caching, output-based conditions, and `for_each` loops.

```
YAML workflow → build_dag → topo_sort → PluginRegistry (dlopen) → run each step
```

See [docs/architecture.md](docs/architecture.md) for internals.

## Build

```bash
# Engine + CLI
cargo build --release

# Plugins (built in-place; the registry discovers them from each
# plugin's target/release/ or from the plugins/ directory)
bash scripts/build-plugins.sh
```

## Quick start

```bash
# List the plugins the registry can load
./target/release/lao-cli plugin-list

# Validate, then run a workflow
./target/release/lao-cli validate workflows/test_loop.yaml
./target/release/lao-cli run workflows/test_loop.yaml
```

A workflow is a list of steps. Steps are ordered by their data/dependency edges;
a step references an upstream step's output by its positional id (`step1`, `step2`, …):

```yaml
workflow: "Audio Transcription & Summary"
steps:
  - run: WhisperPlugin          # step1
    input: "meeting.wav"
    cache_key: "whisper_transcription"
    retries: 2
    retry_delay: 1000

  - run: SummarizerPlugin       # step2
    input_from: step1           # receives step1's output as its input
    cache_key: "summary"
```

See [docs/workflows.md](docs/workflows.md) for the full schema (conditions, loops,
parallel levels, caching) and [docs/cli.md](docs/cli.md) for all commands.

## Bundled plugins

| Plugin | Purpose |
|---|---|
| `EchoPlugin` | Returns its input; reference implementation of the ABI |
| `WhisperPlugin` | Transcribes audio by shelling out to a local `whisper.cpp` build |
| `SummarizerPlugin` | Summarizes text via a local HTTP inference endpoint |
| `PromptDispatcherPlugin` | Generates a workflow YAML from a natural-language prompt |
| `FileReadPlugin` | Reads a file from disk and returns its contents |
| `FolderMapPlugin` | Recursively lists files under a directory |
| `JsonExtractPlugin` | Extracts a value from JSON via a `$.a.b[0]` selector |
| `RegexExtractPlugin` | Returns regex matches from text (first input line is the pattern) |
| `ShellCommandPlugin` | Runs a trusted shell command (gated by `LAO_ALLOW_SHELL=1`) |
| `MarkdownReportPlugin` | Formats text into a Markdown report, optionally writing it to disk |

Writing your own plugin is a matter of implementing the C vtable — see
`lao_plugin_api`, [docs/plugin-api.md](docs/plugin-api.md), and the bundled
plugins for examples.


## Configuration

| Variable | Meaning |
|---|---|
| `LAO_PLUGINS_DIR` | Directory the registry scans for plugins (default: `plugins/`) |
| `LAO_CACHE_DIR` | Directory for cached step outputs (default: `cache/`) |
| `RUST_LOG` | Logging filter for CLI/core diagnostics (default: `warn`) |
| `WHISPER_CPP_PATH` | Path to the `whisper.cpp` build used by `WhisperPlugin` |
| `LAO_ALLOW_SHELL` | Compatibility escape hatch for trusted shell workflows |

Production trust policy is configured through `lao.toml`:

```toml
[trust]
allow_filesystem_read = false
allow_filesystem_write = false
allow_filesystem_enumerate = false
allow_shell = false
allow_network = false
allow_subprocess = false
allow_plugins = []
```

## Notes & limitations

- **Plugins run in-process with full host privileges.** The core trust policy
  denies dangerous capabilities by default, but trusted allowed plugins still run
  with host process privileges.
- **Step success is determined by convention:** a step is considered failed if its
  output is empty or begins with `error:`. The ABI has no separate status channel.
- **Scheduling is persistent and manually triggered.** `schedule`,
  `unschedule`, `list-scheduled`, and `run-due` operate on persisted state. There
  is no background daemon.

## License

Apache-2.0 — see [LICENSE](LICENSE).
