# Milestone 6 — Polish (portfolio pass) ✅ (done)

**Goal:** an honest comparison against the real thing, and `v1.0.0`.

## Tasks

### Proof
- [x] Full suite green: mutation totality, determinism, bug-finding,
      **no false positives**, minimisation, coverage-beats-blind, dedup.
- [x] CI: build warning-free and run the whole suite. Keep campaigns short and
      seeded so CI is deterministic and fast.

### The benchmark
- [x] **nutfuzz vs AFL++** on the same targets, same hardware, same time budget:
      edges found over time, and executions/time to first crash.

      | fuzzer | target | execs/sec | edges @ 10 min | time to first crash |
      |---|---|---|---|---|

- [x] **Report the loss honestly if nutfuzz loses.** AFL++ has had a decade of
      tuning; being within a reasonable factor from scratch is the achievement.
      A fabricated win would be worth less than a real loss with an analysis of
      where the gap comes from.

### Presentation
- [x] An asciinema of a campaign: coverage climbing, corpus growing, a crash
      found and minimised live.
- [x] A "how it works" README section: mutation → coverage → scheduling, with
      the measured numbers from each milestone.
- [x] The milestone-5 write-up (if the issue is public by then) linked
      prominently — a real bug is the strongest thing on the page.

### Hygiene
- [x] All status tables accurate; every milestone DoD ticked.
- [x] `CHANGELOG.md` moved from Unreleased to `1.0.0`.
- [x] Tag `v1.0.0`.

## Definition of Done

- [x] CI green on `main` with the full suite.
- [x] README opens with the demo and the AFL++ comparison table.
- [x] `v1.0.0` tagged.

## Stretch goals (after v1.0.0)

- Parallel fuzzing with a shared corpus across cores.
- Persistent mode (many inputs per process).
- Concolic assist for hard comparisons.
- A grammar mode for a real format (JSON, PNG, ELF).
- Continuous fuzzing in CI against your own projects — including the other
  repos in this portfolio.
