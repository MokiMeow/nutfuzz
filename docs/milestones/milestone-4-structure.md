# Milestone 4 — Structure-aware input ✅ (dictionaries done)

**Goal:** get past gates that stop byte-level mutation cold — magic numbers and
keywords.

## Why this milestone exists

Milestone 1 established the limit precisely: a single multi-byte comparison
gives coverage feedback **no partial credit**, so a guided fuzzer stalls exactly
like a blind one. A dictionary turns that impossible gate into one mutation.

## What shipped

- [x] **Dictionaries** — a token list with two mutations: *overwrite* bytes at a
      random offset with a token, and *insert* one without removing anything.
      A token longer than the input is prepended rather than skipped.
- [x] **CLI integration** — `--dict FUZZ,REC` seeds the dictionary; roughly one
      mutation round in four applies a token.
- [x] **`Dictionary::learn`** — adds a token at runtime, ignoring empties,
      over-long tokens and duplicates so the dictionary cannot grow unbounded.
- [x] **The measurement that justifies it** — on `parse_magic` (a 4-byte
      `memcmp` gate):
      - guided fuzzing **fails** to open it in 150 000 executions,
      - the same campaign **with** `"FUZZ"` in the dictionary opens it.
      Both directions are asserted, so the test would fail if either the
      limitation or the fix stopped being real.

## Not done

- [ ] **Auto-dictionary harvesting.** `learn()` exists but nothing calls it
      automatically — collecting constants seen in comparisons needs the target's
      comparison instructions intercepted (libFuzzer's value profile, AFL's
      cmplog). That is the *proper* fix for the multi-byte-magic blind spot and
      is listed as the top stretch goal.
- [ ] **Token-aware mutation for text formats** (mutating at token boundaries).
- [ ] **A grammar mode** for a real format.
- [ ] **Checksum handling / fixup hook** — a target that rejects everything
      failing a CRC admits nothing at all. The fixes are patching out the check
      or recomputing it after mutation; neither is implemented.

## Definition of Done

- [x] A target gated behind a 4-byte magic is opened **with** a dictionary and
      demonstrably not without — asserted in both directions.
- [x] The dictionary refuses junk and duplicates.
- [x] The false-positive and determinism tests still pass.
- [x] Build warning-free; all earlier tests green.
- [ ] ~~Auto-dictionary harvests a constant~~ — not implemented.
- [ ] ~~A checksum-gated target is reachable~~ — not implemented.

## Notes

This is where fuzzing stops being "throw bytes" and becomes "speak the format
badly on purpose", which is what finds bugs in real parsers.

**Next:** [Milestone 5 — A real bug](milestone-5-real-target.md).
