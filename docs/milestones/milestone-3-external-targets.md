# Milestone 3 — External targets ✅ (core done)

**Goal:** be able to fuzz real programs, not just in-process functions — which
means process control and telling a crash apart from a hang.

## What shipped

- [x] **Run a binary** — `ExternalTarget` spawns a process, feeds the input on
      stdin, and classifies how it ended.
- [x] **Correct classification**, which is the part that matters:
  - **Crash** — killed by SIGSEGV / SIGABRT / SIGFPE / SIGILL / SIGBUS.
  - **Hang** — exceeded its timeout. A separate finding class that still needs
    triage to rule out intentional long work.
  - **Exited** — ran to completion. A non-zero status is a program correctly
    *rejecting* bad input, not a finding.
  - **HarnessError** — the binary is missing or could not be spawned. A broken
    harness must never look like a bug.
- [x] **Timeouts** — a target past its budget is killed and recorded as a hang.
- [x] **Campaign CLI** — `--external`, sorted `--corpus`, repeatable `--arg`,
      timeout, seed, iteration, dictionary, and artifact options.
- [x] **Minimised artifacts** — deterministic `.bin` reproducers plus metadata;
      crashes preserve their signal and hangs preserve timeout behavior.
- [x] **Robust stdin and cleanup** — file-backed input cannot deadlock before
      timeout, and Unix timeouts terminate the target process group.
- [x] Tests that force each case: `/bin/true`, `/bin/false`, an infinite `sh`
      loop (must be a hang, not a crash), `kill -SEGV $$` (must be a crash), and
      a nonexistent binary (must be a harness error).

## Not done

- [ ] **Resource limits (`setrlimit`)** — a runaway target should be capped so
      it cannot exhaust host memory. Requires libc bindings or raw syscalls,
      which conflicts with the zero-dependency rule; needs a small `unsafe`
      syscall wrapper. Not implemented, so **do not point this at an
      allocation-happy target unsupervised.**
- [ ] **Fork server** — forking a target already past initialisation instead of
      `exec`ing fresh each time is AFL's headline speed trick (typically ~10×).
      Not implemented; external-target throughput is `exec`-bound.
- [ ] **Coverage from external binaries** — the in-process path records edges;
      an external process needs a shared memory map plus compile-time
      instrumentation of the target. Not implemented, so external targets
      currently fuzz **blind**.
- [x] **Crash artefacts on disk** (`crashes/` with signal + minimised copy).
- [ ] **A campaign against a real external binary.** The execution layer is
      unit-tested; no real program has been fuzzed through it.

## Definition of Done

- [x] Signals are correctly classified; an infinite loop is a **hang**, not a
      crash.
- [x] A missing binary is a harness error, not a finding.
- [x] Large unread inputs cannot bypass the timeout.
- [x] The external campaign is seed-reproducible, stays quiet on `/bin/true`,
      minimises a test signal, and writes stable artifact names.
- [x] Build warning-free; all earlier tests green.
- [ ] ~~Fuzzes a real external binary and finds its bug~~ — the capability is
      built and tested, the campaign has not been run. See
      [milestone 5](milestone-5-real-target.md).
- [ ] ~~Fork-server speedup measured~~ — not implemented.

## Notes

Only fuzz software you own or are authorised to test. From here the tool can
point at third-party code, so §8 of [AGENTS.md](../../AGENTS.md) applies:
findings go through coordinated disclosure, and reproducers for unfixed
third-party bugs stay out of this repository.

**Next:** [Milestone 4 — Structure-aware input](milestone-4-structure.md).
