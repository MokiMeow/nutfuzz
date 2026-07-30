# Testing

## Local gates

```bash
RUSTFLAGS=-Dwarnings cargo build --release --all-targets
RUSTFLAGS=-Dwarnings cargo test --release --all-targets
```

The current suite has 82 passing tests. `cargo fmt` and `cargo clippy` are
additional gates when their optional Rust components are installed.

## Evidence groups

| Group | What it proves |
|---|---|
| Unit tests | mutation totality, random determinism, coverage, corpus, tokens |
| `coverage_wins` | guidance, scheduling, stability, dictionary and fixup value |
| `finds_bugs` | findings, minimisation, replay, corrected-target silence |
| `external_cli` | CLI parsing, clean programs, limits, harness failures |
| `structure` | harvesting, delimiter preservation, balanced JSON |
| `forkserver` | protocol bounds, versions, coverage, crash and timeout recovery |

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
