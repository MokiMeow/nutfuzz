# Milestone 6 — Polish ✅ (done)

**Goal:** an honest benchmark, green CI, and `v1.0.0`.

## What shipped

### Proof
- [x] **56 tests**: mutation totality (empty / one-byte / max-length inputs),
      PRNG determinism, campaign reproducibility, coverage semantics
      (edges ≠ blocks, bucketing, signatures), corpus scheduling and the
      favoured cover, dictionary behaviour, process classification, bug finding,
      **no false positives**, minimisation, and dedup.
- [x] **The measurement**: `--bench` runs blind and guided over ten seeds and
      reports executions-to-first-crash.
- [x] CI: warning-free build, the full suite, and explicit checks that the buggy
      target yields a finding **and** the corrected target yields none.

### The benchmark
- [x] **Blind vs coverage-guided**, same engine, same budget, same seeds:

      | fuzzer | found | median execs |
      |---|---|---|
      | blind mutation | 0/10 | not found |
      | coverage-guided | 8/10 | 126 059 |

- [x] The result reported as-is, including that guided finds it on 8 of 10
      seeds rather than all 10.

### Presentation
- [x] README opens with the benchmark, a real campaign transcript, and the
      minimised 7-byte reproducer with its replay command.
- [x] A "what coverage feedback cannot do" section — the multi-byte-magic blind
      spot, stated plainly with the test that pins it down.

### Hygiene
- [x] Status tables accurate; every milestone's unfinished items marked
      unfinished rather than ticked.
- [x] `CHANGELOG.md` moved to `1.0.0` with a Known Limitations section.
- [x] Tagged `v1.0.0`.

## Not done

- [ ] **A benchmark against AFL++.** The spec asked for it; what exists is
      blind-vs-guided *within* nutfuzz. Comparing against AFL++ needs the same
      target instrumented for both engines, which is real work and would be the
      more impressive number — it is a stretch goal, not a claim.
- [ ] **An asciinema recording** of a live campaign. The README uses transcripts.

## Definition of Done

- [x] CI green on `main` with the full suite.
- [x] README opens with the demo and a measured comparison.
- [x] `v1.0.0` tagged.
- [ ] ~~README opens with the AFL++ comparison~~ — see above.

## Stretch goals (after v1.0.0)

- **Comparison tracing** (cmplog / value profile) — the proper fix for the
  multi-byte-magic blind spot.
- **Fork server** — roughly 10× on external targets.
- Coverage from external binaries via a shared memory map.
- `setrlimit` resource caps.
- Parallel fuzzing with a shared corpus; sanitizer integration; a grammar mode.
- Continuous fuzzing in CI against the other repos in this portfolio.
