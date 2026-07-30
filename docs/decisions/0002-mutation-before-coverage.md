# ADR 0002: Build blind mutation before coverage feedback

**Status:** accepted · **Date:** 2026

## Context

Coverage guidance is what makes a modern fuzzer effective, so the tempting first
milestone is to build it immediately. Milestone 0 instead ships a *blind*
mutation fuzzer with no feedback at all.

## Decision

Ship the mutation engine, fuzzing loop, determinism guarantee and minimiser
first. Add coverage feedback in milestone 1.

## Rationale

- **It makes the improvement measurable.** Milestone 1's Definition of Done is a
  table of executions-to-first-crash, blind versus guided, on a
  nested-condition target. Without a baseline that number does not exist, and
  "coverage guidance helps" stays an assertion.
- The unglamorous parts: determinism, minimisation, no false positives: are
  what make *any* finding credible, and they are easier to get right before a
  feedback loop is moving the corpus underneath them.
- A blind fuzzer is a genuinely useful reference implementation. When guided
  fuzzing later behaves oddly, being able to compare against a simple known-good
  loop is worth a great deal.

## Consequences

- Milestone 0's corpus never grows, so it cannot reach bugs behind several
  nested conditions. This is stated plainly in the README and
  [docs/05](../05-mutation.md) rather than glossed over.
- Milestone 1 must demonstrate a large win on a nested-condition target. If it
  does not, the implementation is wrong: the ADR makes that falsifiable.
