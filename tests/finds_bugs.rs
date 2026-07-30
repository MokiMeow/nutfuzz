//! The tests that decide whether this is a fuzzer or a random byte generator.
//!
//! Three properties, each checked by trying to break it:
//!   1. it finds a planted bug that random bytes would not stumble on,
//!   2. the same seed reproduces the same run exactly,
//!   3. it reports nothing on a target with nothing to find.
//!
//! (3) is the one most easily skipped and the most important: a "fuzzer" that
//! always reports a crash is worthless, and only a clean run on a corrected
//! target makes the other findings mean anything.

use nutfuzz::fuzzer::{fuzz, minimise, Config, Mode};
use nutfuzz::target::{self, Outcome};
use nutfuzz::Coverage;

fn seeds() -> Vec<Vec<u8>> {
    vec![b"REC".to_vec(), b"REC\x01\x00".to_vec()]
}

fn config(seed: u64, iterations: u64, stop: bool) -> Config {
    Config {
        seed,
        max_iterations: iterations,
        stop_on_crash: stop,
        mode: Mode::Guided,
        stability_runs: 1,
        ..Config::default()
    }
}

fn crashes(input: &[u8]) -> bool {
    let mut cov = Coverage::new();
    matches!(target::parse_record(input, &mut cov), Outcome::Crash(_))
}

#[test]
fn finds_the_planted_bug() {
    let report = fuzz(target::parse_record, &seeds(), &config(0xC0FFEE, 300_000, true));
    assert!(
        !report.findings.is_empty(),
        "failed to find the planted bug in {} executions",
        report.executions
    );
    let finding = &report.findings[0];
    assert!(finding.message.contains("out-of-bounds"));
    assert!(finding.input.starts_with(b"REC"));
    assert_eq!(finding.input[3], 0x07, "must reach the buggy record kind");
}

#[test]
fn the_same_seed_reproduces_the_same_run() {
    let a = fuzz(target::parse_record, &seeds(), &config(424_242, 100_000, true));
    let b = fuzz(target::parse_record, &seeds(), &config(424_242, 100_000, true));

    assert_eq!(a.executions, b.executions);
    assert_eq!(a.findings.len(), b.findings.len());
    assert_eq!(a.corpus_size, b.corpus_size, "the corpus must evolve identically");
    if let (Some(x), Some(y)) = (a.findings.first(), b.findings.first()) {
        assert_eq!(x.input, y.input, "the reproducer must be identical");
        assert_eq!(x.iteration, y.iteration);
    }
}

#[test]
fn different_seeds_explore_differently() {
    let a = fuzz(target::parse_record, &seeds(), &config(1, 100_000, true));
    let b = fuzz(target::parse_record, &seeds(), &config(2, 100_000, true));
    assert!(!a.findings.is_empty() && !b.findings.is_empty());
    assert_ne!(
        (a.findings[0].iteration, &a.findings[0].input),
        (b.findings[0].iteration, &b.findings[0].input),
        "independent seeds should not produce identical campaigns"
    );
}

#[test]
fn reports_nothing_when_there_is_nothing_to_find() {
    let report = fuzz(
        target::parse_record_fixed,
        &seeds(),
        &config(7, 200_000, false),
    );
    assert!(
        report.findings.is_empty(),
        "false positive: reported {} crash(es) on a correct target",
        report.findings.len()
    );
    assert_eq!(report.executions, 200_000);
}

#[test]
fn minimised_reproducer_is_small_and_still_crashes() {
    let report = fuzz(target::parse_record, &seeds(), &config(0xBEEF, 300_000, true));
    let finding = report.findings.first().expect("should find the bug");

    let small = minimise(target::parse_record, &finding.input);
    assert!(crashes(&small), "the minimised input must still crash");
    assert!(small.len() <= finding.input.len());
    // The smallest reproducer is 5 bytes: "REC" + kind + a non-zero length.
    assert!(
        small.len() <= 16,
        "should shrink to something readable, got {} bytes",
        small.len()
    );
}

#[test]
fn minimise_leaves_a_non_crashing_input_alone() {
    let harmless = b"hello world".to_vec();
    assert_eq!(minimise(target::parse_record, &harmless), harmless);
}

#[test]
fn campaign_with_no_seeds_still_runs() {
    let report = fuzz(target::parse_record, &[], &config(3, 5_000, false));
    assert_eq!(report.executions, 5_000);
}

#[test]
fn crashes_are_deduplicated_by_root_cause() {
    // Many different inputs trigger the one planted bug. Reporting each as a
    // separate finding is how fuzzers drown the person who has to fix them.
    let report = fuzz(target::parse_record, &seeds(), &config(5, 150_000, false));
    assert!(!report.findings.is_empty(), "should find the bug");
    assert_eq!(
        report.unique_bugs,
        report.findings.len(),
        "findings should already be deduplicated by signature"
    );
    assert!(
        report.unique_bugs <= 2,
        "there is essentially one bug here; got {} unique signatures",
        report.unique_bugs
    );
}

#[test]
fn coverage_is_actually_recorded() {
    let report = fuzz(target::parse_record, &seeds(), &config(9, 20_000, false));
    assert!(
        report.edges_found > 0,
        "a guided campaign must record edges — zero means the feedback is dead"
    );
}
