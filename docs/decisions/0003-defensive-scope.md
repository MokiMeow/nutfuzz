# ADR 0003 — Bug finding, not exploitation

**Status:** accepted · **Date:** 2026

## Context

A fuzzer is dual-use: the same tool that helps a maintainer harden a parser can
be pointed at software the operator has no business testing, and a crash can be
the first step toward an exploit.

## Decision

nutfuzz is scoped as a **bug-finding and hardening tool**. It finds and
minimises crashes; it does not weaponise them. Real-world findings go through
**coordinated disclosure**.

Concretely:

- Fuzz only software you own or are authorised to test.
- Report privately to maintainers first, agree a timeline, publish after a fix.
- **No reproducer for an unfixed third-party bug is committed to this public
  repository**; keep it private until the issue is public.
- No exploit development, no detection evasion, no crash-to-shell tooling.

## Rationale

- It is the professional norm, and the norm exists because premature disclosure
  puts users at risk while maintainers are still fixing the bug.
- It costs the project nothing technically: the interesting engineering is the
  search — mutation, coverage, scheduling — not what a crash could be turned
  into.
- Stating the boundary explicitly makes the project's intent legible to anyone
  who finds it, which matters for a public security tool.

## Consequences

- Milestone 5 treats disclosure as part of the Definition of Done, not an
  afterthought: a finding is not "done" until it has been reported properly.
- `findings/` holds write-ups only for issues that are already public or fixed.
- Impact is assessed honestly — a null-dereference is a robustness bug, not a
  critical vulnerability, and the write-ups say so.
