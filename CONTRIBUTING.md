# Contributing to CDoc

## Setup

```bash
git clone https://github.com/Moguifeng-9119/cdoc.git
cd cdoc
cargo build --release
```

## Architecture

```
src/
├── main.rs            # CLI entry point + health report rendering
├── lib.rs             # Library exports
├── cli.rs             # Clap command definitions
├── config.rs          # Claude Code path detection (~/.claude/...)
├── error.rs           # Error types
├── output.rs          # Terminal output helpers
├── stats.rs           # cdoc stats — one-page overview
├── doctor.rs          # cdoc doctor — full diagnostic scan
├── fs/                # Filesystem abstraction + JSONL reader
├── rules/             # Rule listing & validation
├── skills/            # Skill listing
├── hooks/             # Hook listing & validation
├── agents/            # Agent listing
├── health/
│   ├── model.rs       # Data structures (SessionSummary, HealthReport, signals)
│   ├── session.rs     # JSONL session parser + session finder
│   ├── watch.rs       # Watch mode (stub)
│   └── signals/       # 7 health signal detectors
└── memory/            # Memory management (Phase 3 placeholder)
```

## Adding a new health signal

1. Create `src/health/signals/your_signal.rs` implementing `HealthSignal` trait
2. Register it in `src/health/signals/mod.rs` → `all_signals()`
3. Add tests in `tests/signals_test.rs`

## Running tests

```bash
cargo test                    # All tests
cargo test -- --nocapture     # Show output
cargo clippy                  # Lint
cargo fmt -- --check          # Format check
```

## Release

```bash
git tag v0.1.0
git push --tags
# GitHub Actions will build and attach binaries to the release
```
