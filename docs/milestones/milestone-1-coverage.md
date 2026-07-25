# Milestone 1 — Coverage feedback

**Goal:** keep inputs that reach new code, so the fuzzer solves one condition at
a time instead of needing to hit them all at once. This is the milestone that
makes it a *real* fuzzer.

## Concepts

Edge coverage, the AFL bitmap, hit-count bucketing, and the corpus as a growing
set of interesting inputs.

## Tasks

- [ ] **A harder target**: add `parse_nested` with a bug behind 3–4 nested
      conditions (magic bytes, then a version check, then a length check).
      Confirm the M0 blind fuzzer **cannot** find it in a large budget — that
      failure is the baseline this milestone must beat.
- [ ] **Instrumentation**: record edges from the target. Start in-process for
      Rust targets (`-C instrument-coverage` or a trace-pc-guard callback);
      external binaries are milestone 3.
- [ ] **Edge bitmap**: a fixed-size byte map indexed by
      `cur_location ^ prev_location`, with `prev_location >>= 1`. Document why
      the XOR identifies an *edge* and why the shift avoids A→B/B→A collisions.
- [ ] **Hit-count buckets** (1, 2, 3, 4–7, 8–15, 16–31, 32–127, 128+) so loop
      iteration counts don't flood the corpus.
- [ ] **The feedback loop**: after each run, if any new edge or new bucket was
      hit, add the input to the corpus.
- [ ] **Non-determinism detection**: run a new input twice; if the bitmaps
      differ, flag it rather than letting it pollute the corpus forever.
- [ ] **Collision metric**: report bitmap fill rate so collisions are visible.
- [ ] **The measurement** (this *is* the milestone): executions-to-first-crash
      on the nested target, median of 10 seeds, blind vs coverage-guided.

## Files

`src/coverage.rs`, `src/fuzzer.rs` (feedback in the loop), `src/target.rs`
(nested target), `benches/` or `tests/coverage_wins.rs`, `docs/06-coverage.md`.

## Definition of Done

- [ ] The nested-condition bug is found by the coverage-guided fuzzer and
      **not** by the blind one within the same budget — asserted by a test.
- [ ] A published table with real numbers:

      | fuzzer | executions to first crash (median of 10 seeds) |
      |---|---|

- [ ] The corpus demonstrably grows as new edges are found (assert it).
- [ ] The false-positive test still passes on the fixed target.
- [ ] Determinism still holds: same seed, same run.
- [ ] Build warning-free; all earlier tests green.

## Notes

If coverage guidance does not beat blind fuzzing by a wide margin on a
nested-condition target, the implementation is wrong — check that edges are
actually being recorded and that the corpus is actually growing. A "coverage
fuzzer" whose corpus never grows is just a slower blind fuzzer.

**Next:** [Milestone 2 — Corpus scheduling](milestone-2-scheduling.md).
