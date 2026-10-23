# Dev commands

Everyday commands for working on gyeoldaero. Keep this current: any PRD that adds a command or
changes how to run something updates this file.

## Checks (what CI runs)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Running against the fixture notebook

```sh
# report problems: exit 0 clean or warnings only, 1 errors, 2 unusable root or config
cargo run -p gyeoldaero-cli -- check tests/fixtures/notebook   # clean: 0 errors, 0 warnings in 23 notes
cargo run -p gyeoldaero-cli -- check tests/fixtures/broken     # known problems, exits 1

# once PRD-002 ships
cargo run -p gyeoldaero-cli -- build tests/fixtures/notebook --out target/site

# once PRD-005 ships
cargo run -p gyeoldaero-cli -- serve tests/fixtures/notebook --bind 127.0.0.1:4321
```

To try it on a phone over the tailnet, bind to the machine's Tailscale address or use
`tailscale serve 4321`.

## Performance check (PRD-001 AC10)

```sh
# loads 5,000 generated notes; must take under 1 s
cargo test --release -p gyeoldaero-core --test perf -- --ignored --nocapture
```

## Snapshot tests

```sh
cargo insta test --workspace     # run and collect new snapshots
cargo insta review               # review each change; accept only what you understand
```

Install with `cargo install cargo-insta` if missing.

## e2e tests (once PRD-003 ships)

```sh
cd tests/e2e
npm ci
npx playwright install --with-deps chromium webkit
npm test
```

## Testing hooks

Each hook reads a JSON payload on stdin, like Claude Code sends. Run from the repo root:

```sh
# format hook: should rustfmt the file and exit 0
echo '{"tool_input":{"file_path":"crates/gyeoldaero-core/src/lib.rs"}}' | .claude/hooks/format-rust.sh; echo "exit $?"

# bash guard: should exit 2 with a message (blocked)
echo '{"tool_input":{"command":"git push --force origin main"}}' | .claude/hooks/guard-bash.sh; echo "exit $?"

# bash guard: should exit 0 (allowed)
echo '{"tool_input":{"command":"cargo test"}}' | .claude/hooks/guard-bash.sh; echo "exit $?"

# stop checks: runs fmt/clippy/tests if Rust files changed
echo '{"stop_hook_active":false}' | .claude/hooks/stop-checks.sh; echo "exit $?"

# session start: prints branch and PRD status
echo '{}' | .claude/hooks/session-start.sh
```

Hooks need `jq` and `git` on `PATH`.
