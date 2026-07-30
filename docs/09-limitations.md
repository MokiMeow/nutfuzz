# Limitations

- One opaque multi-byte comparison provides no partial coverage signal.
  Dictionaries and fixups help known formats, but compiler-assisted comparison
  tracing is not included.
- Arbitrary external binaries do not publish coverage to NutFuzz. The
  coverage-bearing protocol is cooperative and currently demonstrated only by
  the included synthetic harness.
- The JSON generator intentionally covers a small valid subset.
- Printable-token harvesting sees successful input text, not hidden constants
  inside machine instructions.
- Stability checks compare repeated coverage snapshots and cannot identify
  every kind of nondeterministic target behavior.
- Resource limits and the owned forkserver require Linux.
- Exec-per-run external campaigns are slower than instrumented persistent
  fuzzers.
- No sanitizer integration, parallel corpus synchronization, or general
  compiler instrumentation is included.
- No third-party fuzzing campaign was run and no external vulnerability is
  claimed.
