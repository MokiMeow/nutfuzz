# 04 — Roadmap

From "finds a planted bug" to "coverage guidance finds bugs blind fuzzing
provably cannot" — measured.

## The plan, as built

| # | Milestone | What was built | What it taught |
|---|-----------|----------------|----------------|
| 0 | **Mutation core** ✅ | seeded RNG, mutation engine, fuzz loop, minimiser | how mutation fuzzers work; determinism as a contract |
| 1 | **Coverage** ✅ | edge bitmap, bucketing, the feedback loop | why guidance beats blind — and where it doesn't |
| 2 | **Scheduling** ✅ | corpus energy, greedy set-cover favoured set, crash dedup | where fuzzing time should go |
| 3 | **External targets** ✅ | deterministic CLI campaigns, process-tree timeout, minimised artifacts | crashes and hangs are different bugs |
| 4 | **Dictionaries** ✅ | token insert/overwrite, auto-learning | getting past gates coverage can't open |
| 5 | **A real bug** ◐ | the harness and workflow — no campaign run | see below |
| 6 | **Polish** ✅ | benchmark, CI, `v1.0.0` | honest reporting |

## The headline measurement

`parse_nested` hides a bug behind seven gates. 300 000 executions, 10 seeds:

| fuzzer | found | median executions to first crash |
|---|---|---|
| blind mutation | **0/10** | not found |
| coverage-guided | **8/10** | 126 059 |

Blind fuzzing burned three million executions and found nothing.

## Milestone 5 — deliberately unfinished, and why

The **capability** is built and tested: run an external binary, classify a
crash signal against a hang against a clean rejection, enforce a timeout,
minimise, deduplicate by root cause, seed a dictionary.

What has **not** happened is a real campaign against real third-party software,
a real defect, and a real coordinated disclosure. That is not a coding task —
it is choosing an appropriate target, building a harness, running for hours
under sanitizers, triaging, and corresponding with a maintainer.

Marking it ✅ would be the exact overclaim this project's rules forbid, so it is
marked ◐ with the workflow written down in
[milestone 5](milestones/milestone-5-real-target.md) for whoever runs it.

The July 2026 audit intentionally did not launch one: the owner requested
normal safe engineering work only. The external runner itself was completed
and regression-tested without targeting third-party software.

## Engineering capability — met; whole-project DoD still pending

nutfuzz uses coverage feedback to find bugs a blind mutation fuzzer provably
cannot, demonstrates the difference with measured executions-to-first-crash
across ten seeds, runs against external binaries, reports zero false positives
on corrected targets, and reproduces every finding from a seed.

The original whole-project Definition of Done in `AGENTS.md` additionally
requires a genuine authorised third-party finding and responsible disclosure.
That remains unmet with milestone 5 and is not implied by the completed engine.

## Stretch goals (after v1.0.0)

- **Comparison tracing** (cmplog / value profile) — the direct fix for the
  multi-byte-magic blind spot, better than a hand-written dictionary.
- Fork server: fork past initialisation instead of `exec`ing fresh — typically
  an order of magnitude on external targets.
- Parallel fuzzing with a shared corpus across cores.
- Sanitizer integration (ASan/UBSan) so silent memory errors become loud.
- Structure-aware grammars for a real format (JSON, PNG, ELF).
- Continuous fuzzing in CI against the other repos in this portfolio.
