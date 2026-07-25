# 00 — Overview

## What nutfuzz is

A fuzzer: a program that finds bugs by feeding a target malformed input until
something breaks. Written from scratch in Rust with no dependencies — the PRNG,
the mutation engine, the corpus scheduler, the coverage feedback loop and the
crash minimiser are all here.

## The one-sentence idea

> Generate input that breaks things, notice when you have reached somewhere new,
> and keep what got you there.

## Design goals

1. **Determinism is the contract.** Every decision comes from a seeded PRNG, so
   `--seed N` replays a campaign exactly. A fuzzer that cannot reproduce its own
   crash cannot produce a bug report.
2. **No false positives, ever.** A "fuzzer" that always reports something is
   worthless. The suite proves silence on a corrected target, and that test is
   load-bearing for the credibility of every real finding.
3. **A finding is a minimised reproducer**, not a blob. An 8-byte crash input
   shrunk to the 5 bytes that matter is the difference between a report someone
   can act on and noise.
4. **No dependencies.** Writing the search *is* the project
   ([ADR 0001](decisions/0001-no-dependencies.md)).
5. **Measure, don't assert.** Claims that a strategy finds bugs faster come with
   executions-to-first-crash numbers across seeds.

## What it is *not*

- Not an exploitation framework. It finds crashes; it does not weaponise them.
- Not (yet) coverage-guided — that is milestone 1, deliberately sequenced so the
  improvement can be **measured** against the milestone-0 baseline
  ([ADR 0002](decisions/0002-mutation-before-coverage.md)).
- Not a replacement for AFL++/libFuzzer. Milestone 6 benchmarks against them and
  reports the result honestly, win or lose.

## The stack

```
        a real bug, responsibly disclosed             M5
                        │
        structure-aware input: dictionaries, grammar  M4
                        │
        external targets: processes, fork server      M3
                        │
        corpus scheduling: energy, dedup              M2
                        │
        coverage feedback: edge bitmap                M1
                        │
        mutation core: rng, mutations, loop        ✅ M0
```

Milestone 0 is complete: it finds a planted length-field bug in ~3,000
executions, minimises the reproducer to 5 bytes, and reports **nothing** across
200,000 executions of the corrected parser.

Read the [architecture doc](02-architecture.md) next, or
[getting started](01-getting-started.md) to run it.
