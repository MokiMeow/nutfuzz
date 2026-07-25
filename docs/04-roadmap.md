# 04 — Roadmap

From "finds a planted bug" (today) to "found a real bug in real software."

## The plan

| # | Milestone | You'll build | You'll learn |
|---|-----------|--------------|--------------|
| 0 | **Mutation core** ✅ | seeded RNG, mutation engine, fuzz loop, minimiser | how mutation fuzzers actually work; determinism as a contract |
| 1 | **Coverage** | instrumentation, edge bitmap, feedback loop | why coverage guidance beats blind mutation, measurably |
| 2 | **Scheduling** | corpus energy, favoured entries, crash dedup | where fuzzing time should go |
| 3 | **External targets** | run real binaries, fork server, timeouts | process control, speed, hang detection |
| 4 | **Smarter input** | dictionaries, structure-aware mutation | getting past magic bytes and checksums |
| 5 | **A real bug** | fuzz a real library, triage, disclose | the actual job |
| 6 | **Polish** | benchmarks vs AFL++, CI, `v1.0.0` | honest comparison |

## Dependency order

```
M0 ─► M1 ─► M2 ─► M3 ─► M4 ─► M5 ─► M6
```

Linear: scheduling needs coverage to schedule *on*; external targets need the
loop to be fast; structure-aware mutation is only worth it once coverage can
tell you it helped; a real bug needs all of it.

## The measurement that defines success

Milestone 1's whole point is a number, not a feature:

| fuzzer | executions to first crash (median of 10 seeds) |
|---|---|
| blind mutation (M0) | *measured* |
| coverage-guided (M1) | *measured* |

On a target with a bug behind several nested conditions, coverage guidance
should win by orders of magnitude. If it doesn't, the implementation is wrong —
and that is exactly the kind of claim this project insists on proving rather
than asserting.

## Definition of Done (whole project)

nutfuzz finds bugs a blind fuzzer cannot, shows the difference with measured
executions-to-first-crash across seeds, runs against real external binaries, and
has found **at least one genuine bug in real third-party code**, responsibly
disclosed.

## The headline artifact

A short write-up: the bug, how the fuzzer reached it, the minimised reproducer,
and the disclosure timeline. A real finding — even a modest one in a small
library — says more than any benchmark.

## Stretch goals (after v1.0.0)

- Persistent-mode fuzzing (many inputs per process).
- Parallel fuzzing with a shared corpus across cores.
- Sanitizer integration (ASan/UBSan) to catch silent memory errors.
- Concolic/symbolic assist for hard comparisons (magic values, checksums).
- Structure-aware grammars for a real format (JSON, PNG, ELF).
