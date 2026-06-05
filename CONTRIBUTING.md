# Contributing

Pull requests are welcome. For large changes, open an issue or discussion first
to talk through what you're planning.

## Setup

```bash
git clone https://github.com/Moguifeng-9119/cdoc.git
cd cdoc
cargo build --release
```

You need a Rust toolchain (1.80+). The binary ends up at `target/release/cdoc`.

## Layout

```
src/
├── main.rs            # CLI entry point + health report formatting
├── lib.rs             # Library root
├── cli.rs             # clap command definitions
├── config.rs          # Claude Code path detection
├── error.rs           # Error types
├── output.rs          # Terminal helpers
├── stats.rs           # cdoc stats
├── doctor.rs          # cdoc doctor
├── fs/                # File system abstraction + JSONL reader
├── rules/             # Rule listing + validation
├── skills/            # Skill listing
├── hooks/             # Hook listing + validation
├── agents/            # Agent listing
├── health/
│   ├── model.rs       # Data types
│   ├── session.rs     # JSONL parser + session finder
│   ├── watch.rs       # Watch mode (stub)
│   └── signals/       # Seven health signal detectors
└── memory/            # Memory management (placeholder)
```

## Adding a signal

1. Create `src/health/signals/your_signal.rs`, implement `HealthSignal`.
2. Register it in `src/health/signals/mod.rs` → `all_signals()`.
3. Add tests in `tests/signals_test.rs`.

The trait has four methods:

```rust
pub trait HealthSignal {
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn weight(&self) -> f64;
    fn analyze(&self, summary: &SessionSummary) -> SignalResult;
}
```

## Testing

```bash
cargo test                        # All tests
cargo test -- --nocapture         # With output
cargo test signals_test           # Just the signal tests
```

Before pushing:

```bash
cargo fmt -- --check
cargo clippy -- -W clippy::all
cargo test
```

## Release

Tag a version and push. GitHub Actions handles the rest — multi-platform build,
test, and binary upload.

```bash
git tag -a v0.2.0 -m "v0.2.0"
git push --tags
```
