# Milestone 6: Polish (complete)

**Goal:** reproducible evidence, honest comparisons, clear documentation, and
green automation.

## Proof

- [x] 82 release tests cover mutation totality, deterministic random choices,
      coverage semantics, scheduling, progress, stability, trimming,
      dictionaries, grammar generation, checksum fixups, process
      classification, resource controls, protocol validation, forkserver
      recovery, finding discovery, minimisation, and corrected-target silence.
- [x] `--bench` compares blind and guided campaigns across ten fixed seeds.
- [x] Weighted and round-robin scheduling are compared on the same fixed seed,
      target, and budget.
- [x] `examples/forkserver-bench.rs` compares 1,000 persistent owned-target
      runs with 1,000 fresh benign processes.
- [x] CI denies warnings, builds release artifacts, runs the suite, proves the
      planted bug is found, proves the fixed target is quiet, and checks a clean
      external program.

## Presentation

- [x] README documents architecture, requirements, options, measurements,
      limitations, responsible use, and all major documents.
- [x] `docs/assets/campaign.cast` contains authentic fixed-seed checksum
      campaign output in asciinema v2 format.
- [x] `scripts/benchmark-aflpp.sh` builds only the included synthetic C target,
      uses fixed seeds and a bounded time, stores results under `build/aflpp`,
      and exits successfully with an explicit `SKIP` when AFL++ is absent.
- [x] The prepared environment reported that honest skip, so no AFL++ speed
      claim is made.
- [x] Every first-party Markdown file contains zero em dash characters.

## Measured results

| Comparison | Result |
|---|---|
| Blind vs guided, ten seeds | 0/10 vs 8/10 findings |
| Guided median | 126,059 executions |
| Weighted, seed 12648430 | finding at iteration 287,266 |
| Round-robin, same seed | finding at iteration 198,674 |
| Fresh process, 1,000 runs | 1.862 seconds |
| Local forkserver, 1,000 runs | 0.238 seconds |

Measurements describe these exact local workloads and are not universal
performance claims.

## Definition of Done

- [x] Complete warning-denied release build and test suite.
- [x] Deterministic local campaign evidence.
- [x] Safe external-execution and forkserver evidence.
- [x] Optional owned-target AFL++ comparison path.
- [x] Authentic demo artifact.
- [x] Accurate README, roadmap, milestone documents, and changelog.
- [x] Release version `v1.2.0`.

## Remaining optional work

- Compiler-assisted comparison tracing.
- General coverage instrumentation for arbitrary external binaries.
- Parallel workers with corpus synchronization.
- Sanitizer integration for native third-party targets.

The separate third-party campaign and disclosure milestone remains excluded
until a specific target and authorization are supplied.
