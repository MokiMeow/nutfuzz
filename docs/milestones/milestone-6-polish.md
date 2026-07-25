# Milestone 6 — Polish (portfolio pass)

**Goal:** an honest comparison against the real thing, and `v1.0.0`.

## Tasks

### Proof
- [ ] Full suite green: mutation totality, determinism, bug-finding,
      **no false positives**, minimisation, coverage-beats-blind, dedup.
- [ ] CI: build warning-free and run the whole suite. Keep campaigns short and
      seeded so CI is deterministic and fast.

### The benchmark
- [ ] **nutfuzz vs AFL++** on the same targets, same hardware, same time budget:
      edges found over time, and executions/time to first crash.

      | fuzzer | target | execs/sec | edges @ 10 min | time to first crash |
      |---|---|---|---|---|

- [ ] **Report the loss honestly if nutfuzz loses.** AFL++ has had a decade of
      tuning; being within a reasonable factor from scratch is the achievement.
      A fabricated win would be worth less than a real loss with an analysis of
      where the gap comes from.

### Presentation
- [ ] An asciinema of a campaign: coverage climbing, corpus growing, a crash
      found and minimised live.
- [ ] A "how it works" README section: mutation → coverage → scheduling, with
      the measured numbers from each milestone.
- [ ] The milestone-5 write-up (if the issue is public by then) linked
      prominently — a real bug is the strongest thing on the page.

### Hygiene
- [ ] All status tables accurate; every milestone DoD ticked.
- [ ] `CHANGELOG.md` moved from Unreleased to `1.0.0`.
- [ ] Tag `v1.0.0`.

## Definition of Done

- [ ] CI green on `main` with the full suite.
- [ ] README opens with the demo and the AFL++ comparison table.
- [ ] `v1.0.0` tagged.

## Stretch goals (after v1.0.0)

- Parallel fuzzing with a shared corpus across cores.
- Persistent mode (many inputs per process).
- Concolic assist for hard comparisons.
- A grammar mode for a real format (JSON, PNG, ELF).
- Continuous fuzzing in CI against your own projects — including the other
  repos in this portfolio.
