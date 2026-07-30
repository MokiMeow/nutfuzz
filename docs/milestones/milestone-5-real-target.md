# Milestone 5: Authorized third-party campaign (excluded)

**Status:** intentionally not executed.

This milestone is operational security work, not unfinished normal engine
implementation. The user authorized safe local engineering and explicitly
excluded major cyber activity. No named third-party target or written
authorization was supplied, so NutFuzz did not launch a campaign, claim a
vulnerability, contact a maintainer, or publish a reproducer.

## Entry conditions

A future operator may begin this milestone only after all of the following are
true:

1. The exact target and version are named.
2. The operator owns the target or has explicit permission to fuzz it.
3. The target's security policy and reporting channel are documented.
4. CPU, memory, file, time, storage, and network boundaries are agreed.
5. The campaign environment is isolated from production systems and data.

## Authorized workflow

Once the entry conditions are satisfied:

1. Build the smallest target-specific harness.
2. Compile with appropriate sanitizers and debug symbols.
3. Assemble a minimal valid seed corpus.
4. Run a bounded local campaign within the agreed limits.
5. Reproduce, deduplicate, minimize, and manually triage every candidate.
6. Assess impact without inflating severity.
7. Report a genuine issue privately through the documented channel.
8. Publish technical details only after the issue is fixed or already public.

## Evidence required for completion

- Written target authorization.
- Exact source revision and build command.
- Harness and seed-corpus provenance.
- Sanitizer and campaign configuration.
- Stable minimized reproducer for a genuine defect.
- Evidence that corrected code no longer reproduces it.
- Private report and maintainer coordination record.
- Public write-up only when disclosure is appropriate.

None of this evidence exists in this repository, so the milestone remains
excluded rather than falsely marked complete.

## Safe work completed around it

The engine supports deterministic external campaigns, private artifacts,
process-tree timeout cleanup, Linux resource limits, result classification,
minimisation, and a repository-owned local forkserver. All external tests use
synthetic targets or benign system programs.

See [safe campaign and disclosure](../10-disclosure.md).
