<h1 align="center">nutfuzz</h1>

<p align="center">
  <em>A coverage-guided fuzzer written from scratch in Rust — mutation engine,
  edge-coverage feedback, corpus scheduling, dictionaries, and a crash
  minimiser. No dependencies.</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/lang-Rust-orange" alt="Rust">
  <img src="https://img.shields.io/badge/deps-none-brightgreen" alt="no dependencies">
  <img src="https://img.shields.io/badge/reproducible-seeded-blue" alt="deterministic">
  <img src="https://img.shields.io/badge/release-v1.1.0-brightgreen" alt="v1.1.0">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="MIT">
</p>

---

## What this is

A fuzzer finds bugs by feeding a target malformed input until something breaks.
The interesting part is *how it chooses what to feed*: mutation strategies,
coverage feedback that notices when an input reached somewhere new, and
scheduling that decides which input to mutate next.

nutfuzz implements all of that from scratch — no `libfuzzer`, no `afl`, no
`rand` crate.

## The result that matters

A bug hidden behind **seven gates** (four magic bytes, a version, a mode, and a
size check). Same mutation engine, same budget, same seeds — the only
difference is whether coverage feedback is on:

```
$ nutfuzz --bench
budget 300000 executions, 10 seeds

  fuzzer                            found       median execs
  blind mutation                     0/10          not found
  coverage-guided                    8/10             126059
```

**Blind mutation never got through** — 3 million executions across ten seeds,
zero findings. Coverage guidance solves the gates one at a time, because
passing each one reaches new code and that input is kept:

```
$ nutfuzz --target parse_nested
  235783 executions in 4.21s  (55988 exec/s)
  edges 17   corpus 9 (9 favoured)   unique bugs 1

  1 unique bug(s):
    reason    integer overflow: size 255 exceeds the 200-byte buffer
    input     85 bytes  "FUZZ\x02B\xff\xff\xff\x7fBB\xb8 \xb8A\x92..."
    minimised 7 bytes  "FUZZ\x02B\xff"
    replay    nutfuzz --target parse_nested --seed 24301 --iterations 235783
```

An 85-byte crash shrunk to the **7 bytes that actually matter**, with a command
that reproduces it exactly.

And the check that makes any of this mean something — the same fuzzer against a
corrected target:

```
$ nutfuzz --target parse_record_fixed --iterations 200000
  200000 executions ... no crashes found
```

**Zero false positives.** A fuzzer that always reports a crash is worthless.

## What coverage feedback cannot do

Worth stating plainly, because it is the most useful thing here to understand.

Coverage guidance works by giving **partial credit**: getting one gate right
reaches a new edge, so that input is kept and refined. But a *single* four-byte
comparison —

```rust
if &data[0..4] != b"FUZZ" { return; }     // all four bytes, one operation
```

— leaks no progress. Three of four bytes correct reaches exactly the same
edges as zero of four, so there is nothing to steer on and a guided fuzzer
stalls precisely like a blind one. nutfuzz keeps a `parse_magic` target that
demonstrates this, and a test proving a **dictionary** opens the gate that
coverage alone cannot. This is why real fuzzers ship dictionaries and
comparison tracing (AFL's cmplog, libFuzzer's value profile).

## Why it is interesting (the depth on show)

- **Edge coverage, AFL-style** — `index = hash(block) ^ prev`, hit counts
  bucketed so loop iteration counts don't flood the corpus. Block ids are
  *hashed* because sequential ids collide catastrophically under XOR — with
  ids 20–30, two different edges landed on the same index and the signal
  vanished. ([docs/06-coverage.md](docs/06-coverage.md))
- **Corpus scheduling** — energy weighted by size and novelty, plus a greedy
  set-cover "favoured" subset that still reaches every known edge.
- **Crash deduplication by root cause** — grouped by coverage signature, not
  input bytes. One bug reached ten ways is one bug.
- **A real mutation engine** — bit/byte flips, interesting integers at 8/16/32
  bits in both endiannesses, arithmetic deltas, block duplication, splicing,
  stacked 1–8 deep. ([docs/05-mutation.md](docs/05-mutation.md))
- **External targets** — run real binaries, and tell a **crash** (SIGSEGV/
  SIGABRT/…) from a **hang** (timeout) from a program correctly rejecting bad
  input with a non-zero exit. Conflating those is a classic mistake.
- **Deterministic external campaigns** — sorted seed corpora, bounded process
  execution, signal/hang-preserving minimisation, and stable artifacts. The
  external path remains blind until shared-memory coverage instrumentation is
  added. ([docs/07-external-targets.md](docs/07-external-targets.md))
- **Determinism as a contract** — every decision comes from one seeded PRNG, so
  `--seed N` replays a campaign byte for byte, including the iteration a crash
  appeared on.

## Quick start

```bash
cargo run --release                                  # fuzz the record parser
cargo run --release -- --target parse_nested         # the seven-gate target
cargo run --release -- --target parse_nested --mode blind   # the baseline
cargo run --release -- --bench                       # blind vs guided, 10 seeds
cargo run --release -- --dict FUZZ                   # with a dictionary
cargo run --release -- --seed 42 --iterations 500000
cargo run --release -- --external /bin/true --iterations 20
cargo test --release                                 # 65 tests
```

## Status — engine complete; authorised real-world campaign pending

| # | Milestone | State |
|---|-----------|-------|
| 0 | Mutation engine, fuzzing loop, minimiser | ✅ done |
| 1 | Coverage instrumentation + feedback | ✅ done |
| 2 | Corpus scheduling, energy, crash dedup | ✅ done |
| 3 | External binaries: signals, hangs, timeouts | ✅ done |
| 4 | Dictionaries + token mutation | ✅ done |
| 5 | Harness for real targets *(see below)* | ◐ capability built |
| 6 | Benchmarks, CI, `v1.0.0` | ✅ done |

**Milestone 5, honestly:** the machinery to fuzz real external software is
built and tested (CLI, deterministic corpus loading, process-tree timeouts,
signal/hang classification, dictionaries, minimisation, artifacts, and dedup).
**No genuine third-party bug has been found or disclosed through it.** The
owner explicitly kept this audit to normal safe engineering, so no external
vulnerability campaign was launched. See
[milestone 5](docs/milestones/milestone-5-real-target.md) and the
[safe campaign workflow](docs/10-disclosure.md).

## Scope and ethics

A bug-*finding* tool for code you own or are authorised to test. Real findings
go through **coordinated disclosure**: report privately to the maintainer,
agree a timeline, publish after a fix. This repository will never carry a
weaponised exploit or a reproducer for an unfixed third-party bug — the value
here is the search technique.

## Repository layout

```
nutfuzz/
├── src/          # rng, mutation, coverage, corpus, exec, external runner
├── tests/        # bug-finding, false positives, and coverage-beats-blind
├── docs/         # mutation, coverage, roadmap, milestones, ADRs
└── Cargo.toml    # zero dependencies
```

## License

MIT — see [LICENSE](LICENSE).
