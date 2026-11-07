# Testing

## Local gates

```bash
RUSTFLAGS=-Dwarnings cargo build --release --all-targets
RUSTFLAGS=-Dwarnings cargo test --release --all-targets
```

Also run debug-mode tests to expose arithmetic overflow and doctests to check
public examples:

```bash
RUSTFLAGS=-Dwarnings cargo test --all-targets
RUSTFLAGS=-Dwarnings cargo test --doc
```

Formatting and lint checks also run in CI:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

## Evidence groups

| Group | What it proves |
|---|---|
| Unit tests | mutation totality, random determinism, coverage, corpus, tokens |
| `coverage_wins` | guidance, scheduling, stability, dictionary and fixup value |
| `finds_bugs` | findings, minimisation, replay, corrected-target silence |
| `external_cli` | CLI parsing, clean programs, limits, harness failures |
| `structure` | harvesting, delimiter preservation, balanced JSON |
| `forkserver` | complete-frame deadlines, trickled bytes, invalid restarts, protocol limits |

Unit tests also exercise blocked socket writes, artifact symlink/hardlink
replacement, restored private permissions, failed-publication cleanup, and
ordinary descendant cleanup after clean/crashed leaders. In-process
regressions cover preserving crash coverage identity during minimization and
length-growing fixups during corpus trimming. These use only owned synthetic
fixtures and local benign processes.

## Deterministic campaign check

Run the same command twice and compare the target, seed, finding iteration,
signature, and bytes. Ignore elapsed time and executions per second because
they depend on the host.

```bash
./target/release/nutfuzz --target parse_nested \
  --seed 12648430 --iterations 300000
```

## Optional AFL++ comparison

```bash
./scripts/benchmark-aflpp.sh
```

The script compiles only `bench/afl_target.c`, uses a bounded time, and stores
all outputs below `build/aflpp`. Missing AFL++ tools produce an explicit `SKIP`
with exit status zero.
