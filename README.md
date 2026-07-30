<h1 align="center">nutfuzz</h1>

<p align="center">
  A deterministic, coverage-guided fuzzer built from scratch in Rust.
  Mutation, scheduling, structured inputs, minimisation, process isolation,
  and a local forkserver, with no third-party crate dependencies.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021-orange" alt="Rust 2021">
  <img src="https://img.shields.io/badge/dependencies-none-brightgreen" alt="No dependencies">
  <img src="https://img.shields.io/badge/tests-82%20passing-brightgreen" alt="82 passing tests">
  <img src="https://img.shields.io/badge/release-v1.2.0-blue" alt="v1.2.0">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="MIT">
</p>

## Overview

NutFuzz is a compact implementation of the core ideas behind modern fuzzers.
It mutates a seed corpus, records edge coverage, retains inputs that discover
new behavior, assigns mutation energy to useful entries, and reduces findings
to small deterministic reproducers.

The project includes two execution paths:

- In-process targets provide fast, explicit edge coverage for teaching,
  experiments, and deterministic regression tests.
- External targets run as isolated subprocesses with timeout, process-group,
  memory, CPU, file-size, artifact, and minimisation controls.

A versioned local forkserver demonstrates persistent initialization and
coverage-bearing replies against the repository-owned synthetic target. It is
intentionally not a general binary-instrumentation system.

## Measured results

### Coverage guidance

`parse_nested` hides a planted bug behind seven gates. With the same mutation
engine, budget, and ten seeds:

| Mode | Findings | Median executions to first finding |
|---|---:|---:|
| Blind mutation | 0/10 | Not found |
| Coverage-guided | 8/10 | 126,059 |

Blind mutation used three million executions without reaching the planted bug.
Coverage guidance keeps partial progress and solves the gates incrementally.

### Scheduling

For seed `12648430` and a 300,000-iteration budget, both schedulers found the
same 17 edges and the same planted bug:

| Scheduler | First finding |
|---|---:|
| Weighted | 287,266 |
| Round-robin | 198,674 |

Round-robin won this seed. NutFuzz reports the comparison rather than claiming
one policy wins every workload.

### Persistent execution

On the local WSL2 environment, 1,000 executions measured by
`examples/forkserver-bench.rs` produced:

| Executor | Elapsed |
|---|---:|
| Fresh `/bin/true` process per run | 1.862 s |
| Owned local forkserver target | 0.238 s |

That is a measured 7.82x speedup for this synthetic comparison. Results vary by
host and target.

## Features

- Deterministic xorshift64* random generator and replayable campaign seeds.
- Ten bounded mutation strategies with stacked havoc and corpus splicing.
- AFL-style edge hashing, hit-count buckets, and coverage signatures.
- Weighted and round-robin scheduling with a greedy favored corpus.
- Live progress callbacks and CLI progress output.
- Coverage stability checks before corpus admission.
- Coverage-preserving corpus trimming.
- Manual dictionaries, learned printable tokens, and token-aware replacement.
- JSON grammar generation and post-mutation fixup hooks.
- A checksum-gated owned target that proves fixups open otherwise closed paths.
- Crash deduplication and greedy reproducer minimisation.
- External process crash, exit, hang, and harness-error classification.
- Linux process-group cleanup and `setrlimit` controls.
- Private deterministic crash artifacts and sorted seed-corpus loading.
- A framed, versioned, bounded local forkserver protocol with timeout recovery.
- An opt-in AFL++ comparison that only compiles the included synthetic target.

## Architecture

```text
seed corpus
    |
    v
scheduler -> mutation / grammar / tokens -> optional fixup
    |                                      |
    |                                      v
    +------------------------------- target execution
                                            |
                              +-------------+-------------+
                              |                           |
                        edge coverage                run result
                              |                           |
                     stability + trimming      dedup + minimisation
                              |                           |
                         growing corpus              artifacts
```

The in-process feedback loop lives in `src/fuzzer.rs`. External execution is
split between `src/exec.rs`, `src/external.rs`, and `src/limits.rs`. The local
persistent protocol is implemented by `src/protocol.rs`,
`src/forkserver.rs`, and `src/bin/nutfuzz-forkserver-target.rs`.

## Requirements

- Rust 2021 compatible `cargo` and `rustc`.
- Linux for process groups, resource limits, and the local forkserver.
- Unix for the external signal-classification tests.
- Optional AFL++ tools, `afl-fuzz` and `afl-clang-fast`, for the owned-target
  comparison script.

The core in-process engine builds on other Rust-supported platforms. Unsupported
operating-system features return an explicit error instead of silently ignoring
the requested protection.

## Quick start

```bash
cargo build --release
cargo test --release --all-targets

# Find the planted record-parser bug.
./target/release/nutfuzz

# Compare blind and guided campaigns.
./target/release/nutfuzz --bench

# Exercise scheduling and live progress.
./target/release/nutfuzz --target parse_nested \
  --schedule round-robin --progress 10000

# Reach the owned checksum gate with a dictionary and fixup.
./target/release/nutfuzz --target parse_checksum \
  --dict CHECK --fixup checksum --seed 41 --iterations 2000

# Run a clean external target with defensive limits.
./target/release/nutfuzz --external /bin/true --iterations 20 \
  --timeout-ms 500 --memory-mb 64 --cpu-seconds 1 \
  --file-size-bytes 1024
```

Structured local options include `--auto-dict`, `--grammar json`,
`--stability-runs N`, and `--trim`. Use `--seed N` to reproduce the same
mutation decisions.

## Local benchmark and demo

```bash
cargo build --release --all-targets
./target/release/examples/forkserver-bench \
  target/release/nutfuzz-forkserver-target

./scripts/benchmark-aflpp.sh
```

The AFL++ script exits successfully with an explicit `SKIP` when the optional
tools are unavailable. It never downloads software or targets third-party
code. An authentic asciinema v2 recording of the checksum campaign is stored
at [docs/assets/campaign.cast](docs/assets/campaign.cast).

## Testing

The release suite contains 82 tests covering:

- mutation totality on empty, one-byte, and maximum-size inputs;
- deterministic random generation, scheduling, campaigns, and artifacts;
- edge hashing, bucket changes, signatures, stability, and corpus trimming;
- dictionaries, learned tokens, JSON generation, and checksum fixups;
- finding discovery, deduplication, minimisation, and corrected-target silence;
- external exits, signals, timeouts, process cleanup, and resource limits;
- protocol truncation, oversize, version rejection, repeated replies, crashes,
  timeout recovery, and deterministic forkserver coverage.

CI builds with `RUSTFLAGS=-Dwarnings`, runs the complete release suite, proves
the planted bug is found, proves the corrected target stays quiet, and checks a
clean external program.

## Limitations

- Coverage feedback cannot infer partial progress inside one opaque multi-byte
  comparison. Dictionaries and fixups help known formats; full comparison
  tracing is not implemented.
- Arbitrary external binaries do not expose edge coverage. They use the safe
  exec-per-run path unless adapted to the repository's cooperative protocol.
- The JSON grammar is deliberately small and is not a complete JSON generator.
- Stability checks detect changing coverage snapshots, not every source of
  target nondeterminism.
- Resource limits and the forkserver are Linux-specific.
- No third-party fuzzing campaign or vulnerability disclosure was performed.
  That work requires explicit target authorization and remains outside this
  safe local engineering scope.

## Project status

| Milestone | Status |
|---|---|
| Mutation core | Complete |
| Coverage feedback | Complete |
| Scheduling and progress | Complete |
| External process execution | Complete |
| Structured input support | Complete |
| Safe local forkserver and polish | Complete |
| Third-party campaign and disclosure | Intentionally excluded |

## Documentation

- [Overview](docs/00-overview.md)
- [Getting started](docs/01-getting-started.md)
- [Architecture](docs/02-architecture.md)
- [Mutation engine](docs/05-mutation.md)
- [Coverage feedback](docs/06-coverage.md)
- [External targets](docs/07-external-targets.md)
- [Testing](docs/08-testing.md)
- [Limitations](docs/09-limitations.md)
- [Disclosure and authorization](docs/10-disclosure.md)
- [Roadmap](docs/04-roadmap.md)
- [Milestone specifications](docs/milestones/README.md)
- [Changelog](CHANGELOG.md)

## Repository layout

```text
nutfuzz/
|-- src/             engine, executors, protocol, CLI, owned targets
|-- tests/           integration and campaign evidence
|-- bench/           owned AFL++ comparison target
|-- examples/        local executor benchmark
|-- scripts/         reproducible, opt-in benchmark helpers
|-- docs/            concepts, decisions, milestones, and demo asset
`-- Cargo.toml       dependency-free Rust package
```

## Responsible use

Only fuzz software you own or are explicitly authorized to test. Treat
findings as potentially sensitive, minimize access to reproducers, and follow
the target's coordinated-disclosure policy. This repository contains synthetic
planted bugs for validation and no claim about a third-party vulnerability.

## License

MIT. See [LICENSE](LICENSE).
