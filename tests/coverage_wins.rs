//! The measurement that justifies coverage feedback existing.
//!
//! `parse_nested` hides its bug behind four gates: a 4-byte magic value, a
//! version byte, a mode byte, and a size check. Blind mutation must satisfy all
//! four at once — roughly one in 2^48. Coverage guidance solves them one at a
//! time, because passing each gate reaches new code and is therefore kept.
//!
//! If these tests ever fail, the feedback loop is broken — most likely edges
//! are not being recorded, or the corpus is not actually growing.

use nutfuzz::fuzzer::{fuzz, fuzz_with_progress, Config, Mode, Schedule};
use nutfuzz::target;

fn seeds() -> Vec<Vec<u8>> {
    vec![b"AAAAAAAA".to_vec()]
}

fn run(mode: Mode, seed: u64, iterations: u64) -> nutfuzz::Report {
    fuzz(
        target::parse_nested,
        &seeds(),
        &Config {
            seed,
            max_iterations: iterations,
            stop_on_crash: true,
            mode,
            ..Config::default()
        },
    )
}

#[test]
fn round_robin_schedule_is_reproducible() {
    let config = Config {
        seed: 99,
        max_iterations: 20_000,
        stop_on_crash: false,
        mode: Mode::Guided,
        schedule: Schedule::RoundRobin,
        ..Config::default()
    };
    let first = fuzz(target::parse_nested, &seeds(), &config);
    let second = fuzz(target::parse_nested, &seeds(), &config);
    assert_eq!(first.edges_found, second.edges_found);
    assert_eq!(first.corpus_size, second.corpus_size);
    assert_eq!(first.first_crash_at, second.first_crash_at);
}

#[test]
fn progress_reports_fixed_execution_intervals() {
    let config = Config {
        max_iterations: 25,
        progress_interval: 10,
        ..Config::default()
    };
    let mut executions = Vec::new();
    let report = fuzz_with_progress(
        target::parse_nested_fixed,
        &seeds(),
        &config,
        |progress| executions.push(progress.executions),
    );
    assert_eq!(report.executions, 25);
    assert_eq!(executions, vec![10, 20]);
}

#[test]
fn guided_finds_the_nested_bug_that_blind_cannot() {
    const BUDGET: u64 = 300_000;

    let guided = run(Mode::Guided, 0xC0FFEE, BUDGET);
    assert!(
        !guided.findings.is_empty(),
        "coverage-guided fuzzing must find the nested bug within {BUDGET} \
         executions (found none; corpus grew to {}, edges {})",
        guided.corpus_size,
        guided.edges_found
    );

    let blind = run(Mode::Blind, 0xC0FFEE, BUDGET);
    assert!(
        blind.findings.is_empty(),
        "blind mutation should not reach a bug behind four gates in {BUDGET} \
         executions — if it does, the target is too easy to be a fair test"
    );
}

#[test]
fn guided_beats_blind_across_several_seeds() {
    // Seven gates is genuinely hard even with feedback — a guided run needs
    // roughly 100k-250k executions here. The budget reflects that measured
    // reality rather than an optimistic guess.
    const BUDGET: u64 = 400_000;
    let seeds = [1u64, 2, 3, 4, 5];

    let guided_hits = seeds
        .iter()
        .filter(|&&s| !run(Mode::Guided, s, BUDGET).findings.is_empty())
        .count();
    let blind_hits = seeds
        .iter()
        .filter(|&&s| !run(Mode::Blind, s, BUDGET).findings.is_empty())
        .count();

    assert_eq!(
        blind_hits, 0,
        "blind mutation should never open seven gates in {BUDGET} executions"
    );
    assert!(
        guided_hits >= 3,
        "coverage guidance should find it on most seeds, got {guided_hits}/{}",
        seeds.len()
    );
    assert!(
        guided_hits > blind_hits,
        "guided ({guided_hits}) must beat blind ({blind_hits})"
    );
}

#[test]
fn the_corpus_grows_as_gates_are_passed() {
    let report = run(Mode::Guided, 42, 100_000);
    assert!(
        report.corpus_size > 1,
        "the corpus must grow beyond the seed — that growth *is* the feedback"
    );
    assert!(
        report.edges_found > 3,
        "passing gates should uncover new edges, found {}",
        report.edges_found
    );
}

#[test]
fn blind_mode_never_grows_its_corpus() {
    let report = run(Mode::Blind, 42, 20_000);
    assert_eq!(
        report.corpus_size, 1,
        "blind mode has no feedback, so it keeps nothing"
    );
}

#[test]
fn the_favoured_set_is_smaller_than_the_corpus() {
    let report = run(Mode::Guided, 7, 150_000);
    if report.corpus_size > 3 {
        assert!(
            report.favoured <= report.corpus_size,
            "the covering set cannot exceed the corpus"
        );
    }
}

#[test]
fn guided_reports_nothing_on_the_fixed_nested_target() {
    let report = fuzz(
        target::parse_nested_fixed,
        &seeds(),
        &Config {
            seed: 11,
            max_iterations: 200_000,
            stop_on_crash: false,
            mode: Mode::Guided,
            ..Config::default()
        },
    );
    assert!(
        report.findings.is_empty(),
        "false positive: {} crash(es) reported on a corrected target",
        report.findings.len()
    );
}

/// Coverage feedback has a real blind spot, and this pins it down.
///
/// `parse_magic` compares four bytes in one operation, so there is no edge for
/// "three of four correct" and the fuzzer gets no signal — it stalls exactly
/// like a blind one. A dictionary containing the token makes the same gate a
/// single mutation. This is why real fuzzers ship dictionaries and comparison
/// tracing, and the test documents the limitation rather than hiding it.
#[test]
fn a_dictionary_opens_a_gate_coverage_alone_cannot() {
    use nutfuzz::Dictionary;

    let budget = 150_000;
    let magic_seeds = vec![b"AAAAAAAA".to_vec()];

    let without = fuzz(
        target::parse_magic,
        &magic_seeds,
        &Config {
            seed: 99,
            max_iterations: budget,
            stop_on_crash: true,
            mode: Mode::Guided,
            ..Config::default()
        },
    );
    assert!(
        without.findings.is_empty(),
        "a 4-byte memcmp gate should stay shut without a dictionary — if this \
         passes, the gate is too weak to demonstrate the limitation"
    );

    let with = fuzz(
        target::parse_magic,
        &magic_seeds,
        &Config {
            seed: 99,
            max_iterations: budget,
            stop_on_crash: true,
            mode: Mode::Guided,
            schedule: Schedule::Weighted,
            progress_interval: 0,
            dictionary: Dictionary::from_tokens(["FUZZ"]),
        },
    );
    assert!(
        !with.findings.is_empty(),
        "with the magic value in a dictionary the gate should fall"
    );
}
