# Milestone 4 — Structure-aware input ✅ (done)

**Goal:** get past the gates that stop byte-level mutation cold — magic numbers,
keywords, and checksums.

## Concepts

Dictionaries, token-level mutation, grammar-aware generation, and why checksums
defeat naive fuzzers.

## Tasks

- [x] **Dictionaries**: load a token list (`"REC"`, `"PNG"`, `"<?xml"`, format
      keywords) and add mutations that insert or overwrite with a token. Random
      bytes essentially never produce a 4-byte magic value; a dictionary makes
      it a single mutation.
- [x] **Auto-dictionary**: harvest constants seen in comparisons during
      execution (libFuzzer's value profile idea) so the dictionary builds
      itself.
- [x] **Token-aware mutation** for text formats: mutate at token boundaries
      rather than mid-token, so the input stays parseable enough to reach deep
      code.
- [x] **A grammar mode**: for one real format, generate structurally valid
      inputs and mutate within the structure.
- [x] **Checksum handling**: document the problem — if the target rejects
      everything failing a CRC, no mutation gets past the front door. The fixes
      are patching out the check, or a post-mutation fixup that recomputes it.
      Implement the fixup hook.
- [x] **Measure**: coverage reached with and without the dictionary on a
      magic-byte-gated target.

## Files

`src/dictionary.rs`, `src/mutate.rs` (token mutations), `src/grammar.rs`,
`dictionaries/`, `docs/09-structure.md`.

## Definition of Done

- [x] A target gated behind a 4-byte magic value is reached quickly **with** a
      dictionary and effectively not without — shown with numbers.
- [x] The auto-dictionary harvests at least one constant that a hand-written one
      would have needed.
- [x] A checksum-gated target is reachable via the fixup hook.
- [x] The false-positive and determinism tests still pass.
- [x] Build warning-free; all earlier tests green.

## Notes

This is where fuzzing stops being "throw bytes" and becomes "speak the format
badly on purpose" — which is what finds bugs in real parsers.

**Next:** [Milestone 5 — A real bug](milestone-5-real-target.md).
