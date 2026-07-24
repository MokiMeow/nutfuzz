# Milestone 1 — Coverage feedback ✅ (done)

**Goal:** keep inputs that reach new code, so the fuzzer solves one gate at a
time instead of needing to hit them all at once. This is the milestone that
makes it a real fuzzer.

## What shipped

- [x] **A harder target** — `parse_nested`, a bug behind **seven** gates (four
      magic bytes checked individually, a version, a mode, a size). Confirmed by
      test that blind mutation cannot reach it.
- [x] **Instrumentation** — in-process targets call `cov.trace(id)` at each
      branch. Production fuzzers get these callbacks from the compiler
      (`-fsanitize-coverage=trace-pc-guard`); doing it by hand keeps the
      machinery visible. The feedback loop is identical either way.
- [x] **Edge bitmap** — 64 KiB, indexed by `hash(block) ^ prev` with
      `prev = hash(block) >> 1`, so A→B and B→A stay distinct.
- [x] **Hit-count buckets** (1, 2, 3, 4–7, 8–15, 16–31, 32–127, 128+).
- [x] **The feedback loop** — an input that hits a new edge or a higher bucket
      is added to the corpus.
- [x] **Collision metric** — `GlobalMap::fill_rate()`.
- [x] **The measurement** — executions-to-first-crash, blind vs guided, median
      of 10 seeds.

## Not done

- [ ] **Non-determinism detection** (run each new input twice, compare bitmaps,
      flag targets whose coverage varies). The in-process targets here are
      deterministic by construction, so it would be untested scaffolding; it
      becomes necessary the moment real external software is fuzzed. Listed
      rather than silently dropped.

## Verified result

300 000 executions, 10 seeds, on `parse_nested`:

| fuzzer | found | median executions to first crash |
|---|---|---|
| blind mutation | **0/10** | not found |
| coverage-guided | **8/10** | **126 059** |

Blind fuzzing burned three million executions and found nothing.

## Definition of Done

- [x] The gated bug is found by the guided fuzzer and **not** by the blind one
      in the same budget — asserted by a test.
- [x] A published table with real numbers (above).
- [x] The corpus demonstrably grows as edges are found (asserted).
- [x] The false-positive test still passes on corrected targets.
- [x] Determinism holds: same seed, same run, same iteration.
- [x] Build warning-free; all earlier tests green.

## What was learned (worth keeping)

**Two real failures, both caught by tests rather than reasoning.**

**1. A multi-byte comparison is opaque to coverage.** The first version of the
target compared the magic with one `!=` on a 4-byte slice. Guided fuzzing then
performed *exactly* like blind — 3 edges, corpus never grew. There is no edge
for "three of four bytes correct", so there is no gradient to climb. That is a
genuine limit of coverage guidance, and it is why dictionaries and comparison
tracing exist. The `parse_magic` target and its tests preserve the lesson.

**2. Sequential block ids collide under XOR.** With ids 20–30, `22^10` and
`23^11` both land on index 28 — two distinct edges recorded as one, and the
signal quietly disappears. AFL assigns *random* compile-time ids for exactly
this reason. Fixed by hashing the id; a test now asserts each correct magic byte
uncovers a new edge.

Both failures were silent: nothing crashed, nothing errored, the fuzzer just
stopped finding things. That is the characteristic failure mode of a broken
coverage map, and the reason this milestone's Definition of Done is a
*measurement* rather than "the code exists".

**Next:** [Milestone 2 — Corpus scheduling](milestone-2-scheduling.md).
