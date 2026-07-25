# 07 — External targets

NutFuzz can run a normal binary as a deterministic, blind target:

```bash
cargo run --release -- \
  --external /path/to/authorised-target \
  --corpus corpus \
  --iterations 1000 \
  --timeout-ms 500 \
  --artifacts crashes
```

`--arg VALUE` may be repeated to supply target arguments. Corpus files are
loaded in sorted path order, every mutation comes from the campaign seed, and
findings are deduplicated by result class. A crash is minimised while
preserving its signal; a hang is minimised while preserving its timeout.
Artifacts contain the minimised bytes and a text record of the seed,
iteration, result, and sizes. Unix artifact files are created with owner-only
permissions (`0600`).

## Classification

- `Crashed`: the process died from SIGILL, SIGABRT, SIGBUS, SIGFPE, or SIGSEGV;
  this is a finding to triage, not automatic proof of a defect.
- `Hang`: the process exceeded its timeout; intentional long work must still be
  ruled out.
- `Exited`: any normal exit, including non-zero rejection of malformed input.
- `HarnessError`: setup or spawn failed. This stops the campaign and is never
  reported as a target finding.

Target stdin is backed by a temporary file. A pipe can fill when a target never
reads input and block the harness before its timeout loop starts. On Unix, each
target runs in a fresh process group, so timeout cleanup kills descendants as
well as the direct child.

## Current boundary

External campaigns are blind because the target does not yet publish compiler
coverage to NutFuzz. They are also `exec`-bound: there is no fork server. Run
only software you own or are explicitly authorised to test, and supervise
targets that may allocate unbounded memory because address-space limits are not
yet installed.
