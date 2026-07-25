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
pick a parent by deterministic energy scheduling
pick another as a splice donor
havoc(input, donor, rng)               ← 1..8 stacked mutations
occasionally apply a dictionary token
run the target
if its coverage is new:
    add it to the corpus and recompute the favoured cover
if it crashed with a new signature:
    record and minimise it
```

Blind mode preserves the milestone-0 baseline: its corpus never grows and every
candidate comes from a seed. Guided mode runs the loop above. External mode
uses the same seeded mutation but remains blind until instrumented targets can
publish a shared coverage map
([ADR 0002](decisions/0002-mutation-before-coverage.md)).

## Components

| File | Role |
|------|------|
| `src/rng.rs` | xorshift64* PRNG — the reproducibility guarantee |
| `src/mutate.rs` | 10 mutations + `havoc` stacking, with a 64 KiB cap |
| `src/coverage.rs` | edge map, hit buckets, global novelty, signatures |
| `src/corpus.rs` | deterministic energy scheduler and favoured cover |
| `src/dictionary.rs` | bounded token insertion and overwrite |
| `src/target.rs` | targets, including a planted bug and its fixed twin |
| `src/fuzzer.rs` | the campaign loop, `Finding`/`Report`, the minimiser |
| `src/exec.rs` | bounded process execution and result classification |
| `src/external.rs` | blind external campaigns, minimisation, corpus/artifacts |
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
| 2 | `corpus.rs`, `fuzzer.rs` — energy, favoured cover, and dedup |
| 3 | `exec.rs`, `external.rs` — bounded external binary campaigns |
| 4 | `dictionary.rs` — bounded token-aware mutation |

The layering holds throughout: everything random still draws from `rng.rs`.
