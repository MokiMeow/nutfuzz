# Milestone 3 — External targets

**Goal:** fuzz real programs, not just in-process Rust functions — which means
process control, speed, and telling a crash apart from a hang.

## Concepts

Process spawning, exit statuses and signals, timeouts, the fork server, and
resource limits.

## Tasks

- [ ] **Run a binary**: feed input via stdin or a temp file, capture the exit
      status. Distinguish clean exit, non-zero exit, **signal** (SIGSEGV,
      SIGABRT, SIGFPE, SIGILL, SIGBUS = crash), and timeout (hang).
- [ ] **Timeouts**: kill a target that exceeds a budget and record it as a hang,
      not a crash. Hangs are real bugs but a different class.
- [ ] **Resource limits**: `setrlimit` for memory and CPU so a runaway target
      cannot take the machine down. A fuzzer that OOMs its host is a bad fuzzer.
- [ ] **Fork server**: fork from a target already past initialisation instead of
      `exec`ing fresh each time. This is AFL's headline speed trick — typically
      an order of magnitude.
- [ ] **Measure the speedup**: execs/sec with plain exec vs the fork server.
- [ ] **Crash artefacts**: write each unique crashing input to `crashes/` with
      the signal and a minimised copy.

## Files

`src/exec.rs`, `src/forkserver.rs`, `src/target.rs` (external target type),
`docs/08-external-targets.md`.

## Definition of Done

- [ ] Fuzzes a real external binary (a small C program with a planted bug is
      fine as the first one) and finds its bug.
- [ ] Signals are correctly classified; a deliberate infinite loop is reported
      as a **hang**, not a crash.
- [ ] Memory limits are enforced — a target that allocates unboundedly is killed
      rather than taking the host with it.
- [ ] Fork-server speedup is **measured** and reported.
- [ ] Crashing inputs are persisted and replayable from disk.
- [ ] Build warning-free; all earlier tests green.

## Notes

Only fuzz software you own or are authorised to test. From here the tool can
point at real third-party code, so §8 of [AGENTS.md](../../AGENTS.md) applies:
findings go through coordinated disclosure, and reproducers for unfixed
third-party bugs stay out of this repository.

**Next:** [Milestone 4 — Structure-aware input](milestone-4-structure.md).
