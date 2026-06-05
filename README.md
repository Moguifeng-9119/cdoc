# cdoc

A diagnostics tool for Claude Code. Two jobs: audit your configuration, and tell
you when your sessions are going off the rails.

It parses the JSONL session logs that Claude Code writes to disk, runs them
through a set of health signals, and gives you a report. No network calls, no
telemetry, doesn't touch the Claude Code API.

## Why

I built this after noticing that long Claude Code sessions degrade in
predictable ways. Compaction fires, context gets squeezed, and instructions from
CLAUDE.md start dropping out of the assistant's responses. The first thing to go
is usually whatever behavioral rule you wrote at the top of your CLAUDE.md.

Most people don't realize this is happening until they've already burned half an
hour arguing with a model that forgot what it was supposed to do. cdoc surfaces
it early.

## Install

```bash
cargo install cc-doctor
```

Or build from source:

```bash
cargo install --git https://github.com/Moguifeng-9119/cdoc
```

Prebuilt binaries are on the [releases page](https://github.com/Moguifeng-9119/cdoc/releases).

## Usage

```bash
# What's installed?
cdoc stats
cdoc doctor

# Is my current session healthy?
cdoc health latest

# What about the last few?
cdoc health project . --limit 5

# Keep an eye on things
cdoc health watch
```

## Commands

### Config management

| Command | What it does |
|---------|-------------|
| `cdoc stats` | Counts rules, skills, agents, hooks, sessions. One screen. |
| `cdoc list rules` | Lists rule categories, file counts, cross-references. |
| `cdoc list skills` | Lists installed skills with descriptions. |
| `cdoc list hooks` | Shows hooks from settings.json and hooks.json. |
| `cdoc list agents` | Lists agent definitions: tool bindings, models. |
| `cdoc validate rules` | Checks that `extends` and `See skill` references resolve. |
| `cdoc validate hooks` | Validates hook syntax and script existence. |
| `cdoc doctor` | Full scan: empty rule dirs, broken skills/agents, bad settings, session stats. |

### Health monitoring

Seven signals run against every session. No setup required — the tool reads your
CLAUDE.md and calibrates its baseline from the first few assistant responses in
each session.

| # | Signal | What it catches |
|:--:|--------|----------------|
| 1 | **Behavioral baseline** | Greeting pattern, language ratio, and markdown usage drifting mid-session |
| 2 | **Compaction frequency** | Token count cliff-drops (>40%), 2+ per session is a warning |
| 3 | **User corrections** | Phrases like "no, I said", "不对", "你忘了", 29 patterns across Chinese and English |
| 4 | **Tool error rate** | `tool_result.is_error` ratio |
| 5 | **Context pressure** | Peak input tokens relative to the model's known limit |
| 6 | **Cache hit ratio** | Cache read vs. write throughput |
| 7 | **Repeated instructions** | Levenshtein distance >85% on user messages |

Health commands:

```bash
cdoc health latest                # Full report for the most recent session
cdoc health session <UUID>        # Specific session
cdoc health project <DIR>         # Batch-analyze a project's sessions
cdoc health project <DIR> --limit 5
cdoc health watch                 # Polls every 10s, prints only on state change
cdoc health report --format json  # Machine-readable output
```

### Example output

```
🔍  Session Health: 1543aad3-...
   ──────────────────────────────────────────────────
   Duration: 1h 52m  |  Messages: 445  |  Tools: 150  |  Compactions: 1
   Tokens: 502K in / 185K out  |  Peak: 72K  |  Model: deepseek-v4-pro

  Signals
  🟢 Behavioral baseline    [1.00]  No drift detected
  🟢 Compaction frequency   [0.80]  1 compaction, normal
  🟢 User corrections       [0.80]  1 correction / 289 turns = 0.3%
  🟡 Tool error rate        [0.60]  15/150 (10.0%) slightly elevated
  🟢 Context pressure       [1.00]  Peak 72K / 1000K (7%) plenty of room

  🟢 Overall: HEALTHY (0.85)
```

## Model coverage

Context window limits are detected from the model name reported in the session
log. Unknown models default to 200K.

| 1M | 400K | 256K | 200K | 128K |
|-----|------|------|------|------|
| Opus 4.7, Sonnet 4.6, DeepSeek V4, MiniMax M3, Qwen Turbo, MiMo Pro, GPT-5.4+ | GPT-5.1–5.3 | Qwen3-235B, Kimi K2, MiMo Flash, GPT-5 | Claude (general), GLM-4.7, GPT-4o, MiniMax M2.7 | DeepSeek V3, Qwen, GLM, Kimi, LLaMA, Mistral |

## Troubleshooting

**"Claude config directory not found"**

cdoc expects `~/.claude/` to exist. If you haven't used Claude Code yet, run
`claude` once to initialize it.

**"No sessions found"**

Sessions live under `~/.claude/projects/`. An empty directory means no sessions
have been recorded.

**Some sessions fail to analyze**

Run `cdoc health session <UUID>` to see the specific error. Damaged JSONL lines
are skipped and reported as "Malformed lines" in the output.

**Windows paths**

cdoc uses the `dirs` crate for path detection. On Windows, the config directory
is `C:\Users\<user>\.claude\`.

## Contributing

Architecture overview and dev setup are in [CONTRIBUTING.md](CONTRIBUTING.md).

Questions and feature discussion go in [GitHub Discussions](https://github.com/Moguifeng-9119/cdoc/discussions).

## License

MIT
