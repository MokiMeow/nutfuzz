<h1 align="center">nutfuzz</h1>

<p align="center">
  <em>A coverage-guided fuzzer written from scratch in Rust — mutation engine,
  corpus scheduler, coverage feedback and crash minimiser. No dependencies.</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/lang-Rust-orange" alt="Rust">
  <img src="https://img.shields.io/badge/deps-none-brightgreen" alt="no dependencies">
  <img src="https://img.shields.io/badge/reproducible-seeded-blue" alt="deterministic">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="MIT">
</p>

---

## What this is

A fuzzer is a program that finds bugs by feeding a target malformed input until
something breaks. The interesting part is *how it chooses what to feed*:
mutation strategies, corpus scheduling, and coverage feedback that steers the
search toward code paths nobody has reached yet.

nutfuzz implements all of that from scratch — no `libfuzzer`, no `afl`, no
`arbitrary` crate. Milestone 0 is the deterministic mutation core.

## Milestone 0 result

```
$ cargo run --release
nutfuzz milestone 0 — deterministic mutation fuzzing

  target     parse_record
  seed       24301
  iterations 200000

  2976 executions in 0.00s  (5943774 exec/s)

  1 crash(es) found:

    iteration 2975
    reason    out-of-bounds read: record claims 67 payload bytes, only 3 present
    input     8 bytes  "REC\x07C\x01\x01\x00"
    minimised 5 bytes  "REC\x07\x01"
    replay    cargo run --release -- --seed 24301 --iterations 2976
```

It found a planted length-field bug in **2,976 executions**, then shrank the
reproducer to the smallest input that still crashes.

And the check that makes those findings mean anything — the same fuzzer against
a **fixed** version of the parser:

```
$ cargo run --release -- --target parse_record_fixed --iterations 200000
  200000 executions in 0.03s  (6344359 exec/s)

  no crashes found
```

**Zero false positives in 200,000 runs.** A "fuzzer" that always reports a crash
is worthless; only a clean run on a correct target proves the findings are real.

## Why it is interesting (the depth on show)

- **A real mutation engine** — bit and byte flips, interesting integers at each
  width and endianness, small arithmetic deltas, block duplication, and splicing
  between corpus entries, stacked AFL-style in a "havoc" round.
  ([docs/05-mutation.md](docs/05-mutation.md))
- **Determinism as a contract** — every decision comes from one seeded PRNG, so
  `--seed N` replays a campaign byte for byte. A fuzzer that cannot reproduce
  its own findings cannot hand you a bug report.
- **Crash minimisation** — greedy chunk deletion shrinks an 8-byte reproducer to
  the 5 bytes that actually matter, which is the difference between a bug report
  someone can act on and a blob.
- **Coverage feedback** (milestone 1) — instrumenting the target so inputs that
  reach new code are kept, turning random search into guided search.
  ([docs/06-coverage.md](docs/06-coverage.md))

## Quick start

```bash
cargo run --release                                    # fuzz the buggy target
cargo run --release -- --target parse_record_fixed     # prove no false positives
cargo run --release -- --seed 12345 --iterations 500000
cargo run --release -- --list
cargo test                                             # 23 tests
```

Needs only a Rust toolchain.

## Status

Milestone 0 (deterministic mutation core) is **done and tested**.

| # | Milestone | State |
|---|-----------|-------|
| 0 | Mutation engine, fuzzing loop, minimiser | ✅ done |
| 1 | Coverage instrumentation + feedback | ⬜ |
| 2 | Corpus scheduling, energy assignment, dedup | ⬜ |
| 3 | Fuzzing external binaries (fork server) | ⬜ |
| 4 | Dictionaries + structure-aware mutation | ⬜ |
| 5 | Point it at a real library and find a real bug | ⬜ |
| 6 | Benchmarks vs AFL++, CI, `v1.0.0` | ⬜ |

The endgame: **find a genuine bug in a real open-source library and report it
responsibly.** See [docs/04-roadmap.md](docs/04-roadmap.md).

## Scope and ethics

This is a bug-*finding* tool for code you own or are authorised to test. Any
real-world finding goes through **coordinated disclosure**: report privately to
the maintainer first, agree a timeline, publish after a fix. The repository will
never carry a weaponised exploit for an unfixed bug — the value here is the
search technique, not an attack.

## Repository layout

```
nutfuzz/
├── src/          # rng, mutation engine, fuzzing loop, targets
├── tests/        # the tests that prove it finds bugs and avoids false positives
├── docs/         # mutation, coverage, minimisation, roadmap, milestones, ADRs
└── Cargo.toml    # zero dependencies
```

## License

MIT — see [LICENSE](LICENSE).
