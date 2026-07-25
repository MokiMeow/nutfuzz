# Milestone 5 — A real bug

**Goal:** point nutfuzz at real third-party software and find a genuine bug —
then handle it properly. This is the milestone that matters.

## Concepts

Target selection, harness writing, sanitizers, triage, and coordinated
disclosure.

## Tasks

- [ ] **Choose a target you are allowed to fuzz.** Good candidates: a small,
      actively maintained open-source C/C++ parsing library (image, archive,
      config, or serialisation format). Check for an existing security policy
      (`SECURITY.md`) and whether it is already in OSS-Fuzz — a target already
      fuzzed for years is a poor first choice.
- [ ] **Write a harness**: the smallest program that feeds bytes to one entry
      point. Small harness, deep coverage.
- [ ] **Build with sanitizers** (ASan + UBSan). Many real memory bugs are silent
      without them — the program happily continues past an overflow. A crash
      under ASan is a real finding a plain run would miss.
- [ ] **Seed corpus**: gather valid sample files; a good corpus is worth more
      than a clever mutator.
- [ ] **Run a long campaign**, then triage: deduplicate by root cause, minimise
      each reproducer, and read the code to understand *why* it crashes.
- [ ] **Assess impact honestly** — a null-dereference in a parser is a
      robustness bug, not a critical vulnerability. Do not inflate it.
- [ ] **Disclose responsibly**: private report to the maintainer with a
      minimised reproducer and analysis, agree a timeline, publish after a fix.

## Files

`harnesses/`, `corpus/` (seed inputs), `findings/` (write-ups **only for
already-public, fixed issues**), `docs/10-disclosure.md`.

## Definition of Done

- [ ] A genuine, reproducible defect found in third-party code, with a minimised
      reproducer and an explanation of the root cause.
- [ ] Reported privately to the maintainers, following their security policy.
- [ ] A write-up published **only once the issue is fixed or public** — covering
      the bug, how the fuzzer reached it, and the disclosure timeline.
- [ ] Earlier tests all still green.

## Ethics — not optional

- Fuzz only software you own or are authorised to test.
- **Never commit a reproducer for an unfixed third-party bug to this public
  repository.** Keep it private until the issue is public.
- No weaponised exploits. The contribution is the search technique and the
  responsible report.
- If a maintainer asks for more time, give it.

Finding a bug is the easy half; handling it well is what makes it worth putting
your name on.

## Audit status — 2026-07-25

The external campaign runner, deterministic corpus loading, minimisation, and
private artifact path are implemented and tested. The owner explicitly limited
this audit to normal safe engineering work, so no third-party fuzzing or
vulnerability workflow was launched. None of the Definition of Done items
above are claimed; [the disclosure workflow](../10-disclosure.md) records the
requirements for a future authorised campaign.

**Next:** [Milestone 6 — Polish](milestone-6-polish.md).
