# Milestone 2: Corpus scheduling ✅ (done)

**Goal:** spend fuzzing time where it pays. Once the corpus grows, *which* entry
you mutate matters as much as how.

## What shipped

- [x] **Per-entry metadata**: input, edges covered, times picked, favoured
      flag.
- [x] **Energy assignment**: favoured entries get 4×, small inputs (<32 bytes)
      2×, bloated ones (>512 bytes) half, with diminishing returns on entries
      already mined heavily. Selection is weighted-random by energy, not
      round-robin.
- [x] **Favoured set**: a greedy set cover computes the minimal subset of the
      corpus that still reaches every known edge (AFL's "favoured" idea), and
      those entries are preferred.
- [x] **Crash deduplication by root cause**: crashes are grouped by *coverage
      signature* (an FNV hash of the edges taken), not by input bytes. One bug
      reached ten ways is reported once.
- [x] **Stats**: corpus size, favoured count, edges found, execs/sec, and
      unique bugs, reported at the end of a campaign.

## Follow-up completed in v1.2.0

- [x] **Corpus trimming** greedily removes chunks while requiring the exact
      same coverage snapshot. Reports include the total bytes removed.
- [x] **Weighted and round-robin comparison.** Both policies are selectable,
      deterministic, and tested. For seed `12648430`, round-robin found the
      planted bug at iteration 198,674 and weighted scheduling at 287,266.
      Both found 17 edges. This is one workload, not a universal scheduler
      claim.
- [x] **Live stats during a run** are available through `fuzz_with_progress`
      and the CLI `--progress N` option.

## Definition of Done

- [x] Ten different inputs triggering the same bug are reported as **one**
      unique crash (asserted by `crashes_are_deduplicated_by_root_cause`).
- [x] The favoured set is a genuine minimal cover (asserted on a hand-built
      corpus where only 2 of 4 entries are needed).
- [x] Energy demonstrably skews selection toward small inputs (asserted:
      >2400/4000 picks).
- [x] The false-positive and determinism tests still pass.
- [x] Build warning-free; all earlier tests green.
- [x] Scheduling and round-robin are compared with a fixed seed and identical
      budget; the result is reported without requiring weighted scheduling to
      win.

## Notes

Deduplication is where honesty lives in a fuzzer. Counting every crashing input
as a separate finding makes the output look impressive and is useless to whoever
has to fix them. Group by root cause, and report the number of *bugs*.

**Next:** [Milestone 3: External targets](milestone-3-external-targets.md).
