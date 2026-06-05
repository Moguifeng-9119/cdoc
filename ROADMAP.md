# Roadmap

## Done

**Phase 1 — Config management.** `stats`, `list`, `validate`, `doctor`. Reads your
`~/.claude/` directory and tells you what's installed, what's broken, what's
misconfigured.

**Phase 2 — Health monitoring.** Seven signals that parse JSONL session logs
and detect when a Claude Code session is degrading. The behavioral baseline
signal auto-calibrates per session by reading your CLAUDE.md. No manual setup.

## Next

**Configurable thresholds.** Let users tune signal sensitivity in
`~/.claude/cdoc.toml`. Right now the thresholds (40% token drop, 85% similarity,
5% error rate) are hardcoded and reasonable for most people, but some users will
want to dial them up or down.

**User-defined canary patterns.** The behavioral baseline handles the common
case, but if you have very specific output requirements ("always include a
timestamp", "never output YAML"), you want to define explicit pattern checks.
This already works via `~/.claude/cdoc.toml` `[[canary]]` entries — needs
documentation and a `cdoc canary add` command.

**File system watch instead of polling.** `cdoc health watch` currently polls
every N seconds. A real file watcher via `notify` would be lighter and more
responsive.

## Maybe later

- **crates.io publish.** Once the API surface stabilizes.
- **Homebrew formula.** Once there's enough demand to justify it.
- **HTML/PDF reports.** `cdoc health report --format html` for sharing.
- **Multi-project dashboard.** A single view across all your Claude Code
  projects.
