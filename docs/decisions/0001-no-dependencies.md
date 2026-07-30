# ADR 0001: No dependencies

**Status:** accepted · **Date:** 2026

## Context

Crates exist for every part of this: `rand` for the PRNG, `arbitrary` for
structured input, `libfuzzer-sys` or `afl` to wrap an existing engine entirely.

## Decision

Write everything with the Rust **standard library only**.

## Rationale

- The dependencies would replace exactly what the project exists to demonstrate.
  Wrapping `libfuzzer-sys` produces a working fuzzer while teaching nothing
  about how fuzzing works.
- `rand` in particular would undermine the core guarantee: reproducibility
  depends on knowing *precisely* how every random decision is made. A
  hand-written xorshift is auditable in twenty lines.
- Zero dependencies means `cargo test` works offline, with no version churn -
  and the whole search is visible in this repo.

## Consequences

- More code, and correctness sits here: which is why the suite tests mutation
  totality (empty, one-byte, max-length) rather than trusting a library.
- The PRNG is not cryptographic. That is fine: it chooses mutations, it does not
  protect anything.
- Milestone 6 benchmarks against AFL++ knowing it has a decade of tuning; the
  result gets reported honestly either way.
