# 10 — Glossary

- **AFL / AFL++** — the coverage-guided fuzzer whose design most modern fuzzers,
  including this one, follow.
- **Corpus** — the set of inputs the fuzzer mutates from. With coverage feedback
  it grows with every input that reaches new code.
- **Coverage feedback** — keeping an input because it reached a previously
  unseen part of the program; what turns random search into guided search.
- **Crash deduplication** — grouping crashing inputs by root cause, so one bug
  reached ten ways is reported once.
- **Dictionary** — a list of tokens (magic bytes, keywords) inserted by
  mutation, so the fuzzer can get past checks random bytes never pass.
- **Edge** — a transition between basic blocks (A→B). Edge coverage
  distinguishes paths that block coverage alone cannot.
- **Energy** — how many mutations a corpus entry earns; the scheduling knob.
- **False positive** — a reported "crash" that is not a real defect. The reason
  the suite must prove silence on corrected code.
- **Fork server** — forking a target already past initialisation instead of
  `exec`ing it fresh; AFL's big speed win.
- **Hang** — a target exceeding its time budget. A finding to triage separately
  from a crash; intentional long-running work must be ruled out.
- **Havoc** — applying a random stack of mutations in one round; what actually
  drives discovery.
- **Hit-count bucketing** — recording *roughly* how often an edge was taken
  (1, 2, 3, 4–7, …) so loop counts don't flood the corpus.
- **Interesting values** — boundary integers (0, 1, 0x7f, 0x80, 0xff, INT_MAX,
  −1) that disproportionately trigger bugs.
- **Minimisation** — shrinking a crashing input to the smallest one that still
  crashes; what makes a finding actionable.
- **Mutation** — a small edit to an input: bit flip, byte flip, insertion,
  deletion, block duplication, splice.
- **Sanitizer (ASan/UBSan)** — instrumentation that turns silent memory errors
  into loud crashes; essential for finding real memory bugs.
- **Seed (corpus)** — the starting inputs a campaign mutates from.
- **Seed (PRNG)** — the number that makes a campaign reproducible. Both senses
  appear in fuzzing; context distinguishes them.
- **Splice** — grafting part of one corpus input into another; crossover.
- **Triage** — turning raw crashes into a small set of understood, deduplicated,
  minimised bug reports.
- **Coordinated disclosure** — reporting a real finding privately to the
  maintainer, agreeing a timeline, and publishing after a fix.
