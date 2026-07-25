//! The tests that decide whether this is a fuzzer or just a random byte
//! generator.
//!
//! Three properties matter, and each is checked by trying to break it:
//!   1. it finds a planted bug that random bytes would not stumble on,
//!   2. the same seed reproduces the same run exactly,
//!   3. it reports nothing on a target with nothing to find.
//!
//! (3) is the one most easily skipped and the most important: a "fuzzer" that
//! always reports a crash is worthless, and only a clean run on a fixed target
//! proves the findings mean something.

use nutfuzz::fuzzer::{fuzz, minimise, Config};
use nutfuzz::target::{self, Outcome};

fn seeds() -> Vec<Vec<u8>> {
    vec![b"REC".to_vec(), b"REC\x01\x00".to_vec()]
}

#[test]
fn finds_the_planted_bug() {
    let config = Config {
        seed: 0xC0FFEE,
        max_iterations: 300_000,
        stop_on_crash: true,
    };
    let report = fuzz(target::parse_record, &seeds(), &config);

    assert!(
        !report.findings.is_empty(),
        "fuzzer failed to find the planted bug in {} executions",
        report.executions
    );

    let finding = &report.findings[0];
    assert!(finding.message.contains("out-of-bounds"));
    // It must actually be a record that reaches the buggy branch.
    assert!(finding.input.starts_with(b"REC"));
    assert_eq!(finding.input[3], 0x07, "must hit the buggy record kind");
}

#[test]
fn the_same_seed_reproduces_the_same_run() {
    let config = Config {
        seed: 424_242,
        max_iterations: 100_000,
        stop_on_crash: true,
    };
    let a = fuzz(target::parse_record, &seeds(), &config);
    let b = fuzz(target::parse_record, &seeds(), &config);

    assert_eq!(a.executions, b.executions, "execution count must match");
    assert_eq!(a.findings.len(), b.findings.len());
    if let (Some(x), Some(y)) = (a.findings.first(), b.findings.first()) {
        assert_eq!(x.input, y.input, "the reproducer must be identical");
        assert_eq!(x.iteration, y.iteration, "found at the same iteration");
    }
}

#[test]
fn different_seeds_explore_differently() {
    let run = |seed| {
        fuzz(
            target::parse_record,
            &seeds(),
            &Config { seed, max_iterations: 100_000, stop_on_crash: true },
        )
    };
    let a = run(1);
    let b = run(2);
    // Both should find it, but almost certainly not at the same iteration.
    assert!(!a.findings.is_empty() && !b.findings.is_empty());
    assert_ne!(
        (a.findings[0].iteration, &a.findings[0].input),
        (b.findings[0].iteration, &b.findings[0].input),
        "independent seeds should not produce identical campaigns"
    );
}

#[test]
fn reports_nothing_when_there_is_nothing_to_find() {
    let config = Config {
        seed: 7,
        max_iterations: 200_000,
        stop_on_crash: false,
    };
    let report = fuzz(target::parse_record_fixed, &seeds(), &config);
    assert!(
        report.findings.is_empty(),
        "false positive: reported {} crash(es) on a correct target",
        report.findings.len()
    );
    assert_eq!(report.executions, config.max_iterations);
}

#[test]
fn minimised_reproducer_is_small_and_still_crashes() {
    let config = Config {
        seed: 0xBEEF,
        max_iterations: 300_000,
        stop_on_crash: true,
    };
    let report = fuzz(target::parse_record, &seeds(), &config);
    let finding = report.findings.first().expect("should find the bug");

    let small = minimise(target::parse_record, &finding.input);

    assert!(
        matches!(target::parse_record(&small), Outcome::Crash(_)),
        "minimised input must still crash"
    );
    assert!(
        small.len() <= finding.input.len(),
        "minimising must not grow the input"
    );
    // The smallest possible reproducer is 6 bytes: "REC" + kind + len + 1 byte.
    assert!(
        small.len() <= 16,
        "reproducer should shrink to something readable, got {} bytes",
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
    let report = fuzz(
        target::parse_record,
        &[],
        &Config { seed: 3, max_iterations: 5_000, stop_on_crash: false },
    );
    assert_eq!(report.executions, 5_000);
}
