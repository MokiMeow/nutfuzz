# Milestones

Each milestone leaves a fuzzer that **builds clean, finds the bugs it should,
and stays quiet on code with no bugs**.

| # | Milestone | State |
|---|-----------|-------|
| 0 | [Mutation core](milestone-0-mutation-core.md) | ✅ done |
| 1 | [Coverage feedback](milestone-1-coverage.md) | ✅ done |
| 2 | [Corpus scheduling](milestone-2-scheduling.md) | ✅ done |
| 3 | [External targets](milestone-3-external-targets.md) | ✅ done |
| 4 | [Structure-aware input](milestone-4-structure.md) | ✅ done |
| 5 | [A real bug](milestone-5-real-target.md) | ◐ harness built, campaign not run |
| 6 | [Polish](milestone-6-polish.md) | ✅ done |

## Every milestone spec has

**Goal · Concepts · Tasks · Files · Definition of Done · References.**

## The loop (from AGENTS.md)

1. Pick the lowest-numbered unfinished milestone.
2. Implement its tasks.
3. Prove it: the buggy target must still yield a minimised reproducer, **and**
   the fixed target must still yield nothing.
4. Measure — executions-to-first-crash across several seeds, not a claim.
5. Update the doc, tick the DoD, update README/CHANGELOG/roadmap.
6. Commit (`type(scope): …`), keep CI green.

## The two rules that matter most

**Determinism.** Every random decision comes from the seeded `Rng`, so `--seed N`
replays a campaign exactly. A fuzzer that cannot reproduce its own finding
cannot file a bug report.

**No false positives.** The `reports_nothing_when_there_is_nothing_to_find` test
must keep passing at every milestone. A fuzzer that always reports a crash
proves nothing — that test is what makes every other finding credible.
