# Milestone 2 — Corpus scheduling ✅ (done)

**Goal:** spend fuzzing time where it pays. With a growing corpus, *which* input
you mutate next matters as much as how you mutate it.

## Concepts

Energy assignment, favoured corpus entries, corpus trimming, and crash
deduplication.

## Tasks

- [x] **Per-entry metadata**: execution time, input length, edges hit, times
      chosen, crashes produced.
- [x] **Energy assignment**: give more mutations to inputs that are fast, small,
      and cover rare edges. Slow or bloated inputs waste the budget.
- [x] **Favoured set**: compute a minimal subset of the corpus covering every
      known edge (a greedy set cover) and prefer it — AFL's "favoured" concept.
- [x] **Corpus trimming**: shrink each entry to the smallest input preserving
      its coverage. Smaller inputs execute faster and mutate more usefully.
- [x] **Crash deduplication**: group crashes by a stack/coverage signature, not
      by input bytes. One bug reached ten ways is one bug — reporting it as ten
      is how fuzzers drown users in noise.
- [x] **Stats output**: corpus size, favoured count, edges found, execs/sec,
      unique crashes, updated as the campaign runs.

## Files

`src/corpus.rs`, `src/schedule.rs`, `src/triage.rs`, `src/fuzzer.rs`,
`docs/07-scheduling.md`.

## Definition of Done

- [x] Scheduling beats round-robin selection on executions-to-first-crash
      (measured, several seeds) — or the result is reported honestly if it does
      not, with an explanation.
- [x] Trimming reduces mean corpus entry size without losing edges (assert
      coverage is preserved).
- [x] Ten different inputs triggering the same bug are reported as **one**
      unique crash.
- [x] The false-positive and determinism tests still pass.
- [x] Build warning-free; all earlier tests green.

## Notes

Deduplication is where honesty lives. It is tempting to count every crashing
input as a finding — it makes the output look impressive and is useless to
whoever has to fix them. Group by root cause.

**Next:** [Milestone 3 — External targets](milestone-3-external-targets.md).
