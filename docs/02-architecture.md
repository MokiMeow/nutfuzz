# 02 — Architecture

```
┌──────────────────────────────────────────────────┐
│ main.rs — CLI, reporting, replay instructions     │
├──────────────────────────────────────────────────┤
│ fuzzer.rs — the loop, crash recording, minimiser  │
├───────────────────────┬──────────────────────────┤
│ mutate.rs — mutations │ target.rs — what to fuzz  │
├───────────────────────┴──────────────────────────┤
│ rng.rs — the single source of randomness          │
└──────────────────────────────────────────────────┘
```

`rng.rs` sits at the bottom on purpose: **every** decision above it draws from
one seeded generator, which is what makes a campaign reproducible.

## The loop

```
pick an input from the corpus          ← scheduling arrives in M2
pick another as a splice donor
havoc(input, donor, rng)               ← 1..8 stacked mutations
run the target
if it crashed and the input is new:
    record it (and minimise on report)
```

Milestone 0 has no coverage feedback, so the corpus never grows — every
candidate is derived from a seed input. That is the baseline milestone 1 must
beat with a measured number
([ADR 0002](decisions/0002-mutation-before-coverage.md)).

## Components

| File | Role |
|------|------|
| `src/rng.rs` | xorshift64* PRNG — the reproducibility guarantee |
| `src/mutate.rs` | 10 mutations + `havoc` stacking, with a 64 KiB cap |
| `src/target.rs` | targets, including a planted bug and its fixed twin |
| `src/fuzzer.rs` | the campaign loop, `Finding`/`Report`, the minimiser |
| `src/main.rs` | CLI, output, replay command |
| `tests/finds_bugs.rs` | finds the bug, reproduces, **and stays silent on clean code** |

## The two targets, and why there are two

`parse_record` has a planted length-field bug — trusting a length before
validating it, the most common real parser flaw. `parse_record_fixed` is the
same parser, corrected.

Keeping both is a design decision, not scaffolding: the fixed one is how the
suite proves the fuzzer does not report false positives. Without it, "found a
crash" carries no information.

## Minimisation

Greedy chunk deletion: try removing a block; if the input still crashes, keep
the deletion; halve the block size and repeat. It reduces the milestone-0
finding from 8 bytes to the true 5-byte minimum, `REC\x07\x01`.

## Where it grows

| Milestone | New component |
|---|---|
| 1 | `coverage.rs` — edge bitmap; the loop keeps interesting inputs |
| 2 | `corpus.rs`, `schedule.rs`, `triage.rs` — energy and dedup |
| 3 | `exec.rs`, `forkserver.rs` — external binaries |
| 4 | `dictionary.rs`, `grammar.rs` — structure-aware input |

The layering holds throughout: everything random still draws from `rng.rs`.
