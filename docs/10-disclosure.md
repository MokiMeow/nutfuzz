# 10: Safe campaign and disclosure workflow

This repository provides defensive testing infrastructure. It does not grant
permission to test any third-party system.

Before a real campaign:

1. obtain explicit authorisation and record the exact target and scope;
2. read the project's security policy and use the requested reporting channel;
3. use a local, isolated build with sanitizers and conservative resource caps;
4. keep raw findings and reproducers outside this public repository.

If a genuine defect appears, first reproduce it, minimise it while preserving
the same failure, rule out a harness error, and assess impact without
exaggeration. Send the maintainer a private report containing the affected
version, build configuration, minimal reproducer, sanitizer output, and root
cause analysis. Publish only after the issue is fixed or already public and the
maintainer's disclosure timeline permits it.

During the July 2026 repository audit, the owner explicitly requested normal
safe engineering work only. No third-party campaign, vulnerability research,
or disclosure was initiated. Milestone 5 therefore remains openly pending
rather than being completed with invented evidence.
