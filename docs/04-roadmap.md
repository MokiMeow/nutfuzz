# Roadmap

NutFuzz's safe local engine is complete. The remaining campaign milestone
requires explicit authorization for a named third-party target and is not
treated as missing engine code.

## Milestones

| # | Milestone | Status | Evidence |
|---|---|---|---|
| 0 | Mutation core | Complete | deterministic mutation, replay, minimisation |
| 1 | Coverage feedback | Complete | guided 8/10, blind 0/10 |
| 2 | Scheduling | Complete | weighted and round-robin, progress, trimming |
| 3 | External execution | Complete | classification, limits, artifacts, local forkserver |
| 4 | Structured input | Complete | dictionaries, learned tokens, JSON, checksum fixups |
| 5 | Authorized real target | Excluded | no named target authorization or campaign |
| 6 | Polish | Complete | 82 tests, CI, benchmarks, demo, documentation |

## Headline coverage result

`parse_nested` hides a planted bug behind seven gates. Across ten seeds and a
300,000-execution budget:

| Fuzzer | Found | Median executions |
|---|---:|---:|
| Blind mutation | 0/10 | Not found |
| Coverage-guided | 8/10 | 126,059 |

This test remains the core proof that the feedback loop contributes real search
power rather than merely existing in the source tree.

## v1.2.0 completion work

- Stability reruns prevent changing coverage from polluting the corpus.
- Coverage-preserving trimming reduces admitted inputs.
- Weighted and round-robin policies are selectable and measured honestly.
- Progress callbacks expose live campaign state.
- Automatic tokens, token mutation, JSON generation, and checksum fixups cover
  normal structured-input gaps.
- Linux resource limits constrain external target damage.
- The repository-owned persistent harness returns deterministic coverage and
  measured 7.82x faster than the local fresh-process comparison.
- The AFL++ script is bounded and local-only; it reports `SKIP` when tools are
  unavailable.
- All first-party Markdown is free of em dash characters.

## Deliberately excluded milestone

No third-party software was fuzzed. No vulnerability, disclosure, or maintainer
response is claimed. Milestone 5 remains a documented workflow because running
it requires:

1. a named target the operator is authorized to test;
2. a target-specific harness and sanitizer build;
3. a bounded campaign and manual triage;
4. private coordinated disclosure if a genuine issue is found.

See [milestone 5](milestones/milestone-5-real-target.md) and
[disclosure guidance](10-disclosure.md).

## Optional future work

- Compiler-assisted comparison tracing.
- General external coverage instrumentation.
- Parallel workers with corpus synchronization.
- Sanitizer integration and richer format grammars.
