# Changelog

All notable changes to nutfuzz are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Milestone 0: the deterministic mutation core — a seeded xorshift64* PRNG that
  every search decision draws from, ten AFL-style mutations (bit and byte flips,
  interesting integers at 8/16/32 bits in both endiannesses, arithmetic deltas,
  insert, delete, block duplication, corpus splice) stacked 1–8 per `havoc`
  round under a 64 KiB length cap.
- A fuzzing loop with crash recording, and a greedy chunk-deletion minimiser
  that shrinks a reproducer to the smallest input that still crashes.
- Two targets: a toy record parser with a **planted length-field bug**, and a
  corrected version of the same parser used to prove the fuzzer does not report
  false positives.
- A CLI with `--target`, `--seed`, `--iterations`, `--keep-going` and `--list`,
  printing a replay command alongside every finding.
- 23 tests: mutation totality on empty/one-byte/max-length inputs, PRNG
  determinism, bug finding, campaign reproducibility, minimisation, and the
  false-positive check.
- Documentation set under `docs/` (mutation engine, coverage plan, architecture,
  glossary), 3 ADRs including the project's defensive scope, 7 milestone specs,
  and the `AGENTS.md` working agreement.

### Verified
- `parse_record`: crash found at iteration 2975 of 2976 executions; 8-byte input
  minimised to the true 5-byte minimum `REC\x07\x01`.
- `parse_record_fixed`: 200,000 executions, **no crashes** — no false positives.

## [0.1.0] — milestone 0
- First working version: a deterministic mutation fuzzer with minimisation.
