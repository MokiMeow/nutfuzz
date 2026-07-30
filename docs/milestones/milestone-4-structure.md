# Milestone 4: Structure-aware input (complete)

**Goal:** cross format gates that byte-level mutation and coverage feedback
cannot solve alone.

## Why this milestone exists

One opaque multi-byte comparison provides no partial coverage credit. A
dictionary turns a known magic value into one mutation. Token boundaries,
grammars, and checksum repair extend that idea to structured inputs.

## What shipped

- [x] Bounded dictionary learning rejects empty, duplicate, and overlong
      tokens.
- [x] Dictionary insertion and overwrite never exceed the 64 KiB input cap.
- [x] `--dict FUZZ,REC` provides manual tokens.
- [x] Automatic harvesting learns unique printable runs of 4 through 32 bytes
      from stable coverage-increasing inputs.
- [x] Token-aware replacement preserves surrounding lexical delimiters.
- [x] `--grammar json` adds a small balanced JSON-object generator as a
      campaign mutation strategy.
- [x] Post-mutation `FixupFn` hooks run before target execution and during
      coverage-preserving trim trials.
- [x] The owned `parse_checksum` target requires a valid wrapping checksum and
      the structured `CHECK` token.

## Verified behavior

- Coverage guidance fails to open the opaque `FUZZ` gate within 150,000
  executions without a dictionary and opens it with the same budget and token.
- Token harvesting is unique and bounded.
- Token replacement changes `true` to `false` without damaging the surrounding
  JSON punctuation.
- 1,000 generated JSON values have balanced objects and quotes.
- The checksum campaign remains quiet without a fixup and reaches its planted
  finding with `fixup_checksum`.

## Definition of Done

- [x] A multi-byte magic gate is opened with a dictionary and demonstrably not
      without it.
- [x] Automatic token harvesting is bounded and tested.
- [x] Token-aware mutation and JSON generation preserve their basic structure.
- [x] A checksum-gated target is reachable only with its tested fixup.
- [x] False-positive and determinism tests remain green.
- [x] Build warning-free; all earlier tests green.

## Limitation

Harvesting printable runs is not compiler-assisted comparison tracing. Cmplog
or value profiling would reveal constants that never appear in successful
inputs and remains a worthwhile future improvement.

**Next:** [Milestone 5: A real bug](milestone-5-real-target.md).
