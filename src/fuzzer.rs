//! The fuzzing loop.
//!
//! Blind mode mutates a fixed seed set and throws every result away. Guided
//! mode adds the one idea that makes modern fuzzers work: if an input reaches
//! an edge never seen before, **keep it** and mutate it further. Solving a
//! nested condition becomes progress the fuzzer can build on, rather than
//! something it must luck into all at once.
//!
//! Both modes are kept so the difference can be *measured* rather than
//! asserted — see `tests/coverage_wins.rs`.

use std::collections::HashMap;
use std::time::Instant;

use crate::corpus::Corpus;
use crate::coverage::{signature, Coverage, GlobalMap};
use crate::dictionary::Dictionary;
use crate::mutate::havoc;
use crate::rng::Rng;
use crate::structure::{
    generate, harvest_tokens, mutate_token, FixupFn, Grammar,
};
use crate::target::{Outcome, TargetFn};
use crate::trim::trim_coverage;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// No feedback: mutate the seeds forever (the milestone-0 baseline).
    Blind,
    /// Keep inputs that reach new edges.
    Guided,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Schedule {
    Weighted,
    RoundRobin,
}

pub struct Config {
    pub seed: u64,
    pub max_iterations: u64,
    pub stop_on_crash: bool,
    pub mode: Mode,
    pub schedule: Schedule,
    pub progress_interval: u64,
    pub stability_runs: u8,
    pub trim_corpus: bool,
    pub auto_dictionary: bool,
    pub grammar: Option<Grammar>,
    pub fixup: Option<FixupFn>,
    pub dictionary: Dictionary,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 0x5EED,
            max_iterations: 200_000,
            stop_on_crash: true,
            mode: Mode::Guided,
            schedule: Schedule::Weighted,
            progress_interval: 0,
            stability_runs: 2,
            trim_corpus: false,
            auto_dictionary: false,
            grammar: None,
            fixup: None,
            dictionary: Dictionary::new(),
        }
    }
}

/// A reproducible crash.
#[derive(Debug, Clone)]
pub struct Finding {
    pub input: Vec<u8>,
    pub message: String,
    pub iteration: u64,
    /// Coverage signature — crashes sharing one are the same bug.
    pub signature: u64,
}

#[derive(Debug)]
pub struct Report {
    pub executions: u64,
    pub elapsed_secs: f64,
    pub findings: Vec<Finding>,
    pub seed: u64,
    pub edges_found: usize,
    pub corpus_size: usize,
    pub favoured: usize,
    /// Distinct crash signatures — the number of *bugs*, not crashing inputs.
    pub unique_bugs: usize,
    /// Iteration of the first crash, if any.
    pub first_crash_at: Option<u64>,
    pub stability: Stability,
    pub trimmed_bytes: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Stability {
    pub checks: u64,
    pub unstable_inputs: u64,
}

impl Report {
    pub fn execs_per_second(&self) -> f64 {
        if self.elapsed_secs <= 0.0 {
            0.0
        } else {
            self.executions as f64 / self.elapsed_secs
        }
    }
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub executions: u64,
    pub corpus_size: usize,
    pub favoured: usize,
    pub edges_found: usize,
    pub unique_bugs: usize,
    pub elapsed_secs: f64,
}

/// Run a campaign.
pub fn fuzz(target: TargetFn, seeds: &[Vec<u8>], config: &Config) -> Report {
    fuzz_with_progress(target, seeds, config, |_| {})
}

pub fn fuzz_with_progress<F>(
    target: TargetFn,
    seeds: &[Vec<u8>],
    config: &Config,
    mut on_progress: F,
) -> Report
where
    F: FnMut(&Progress),
{
    let mut rng = Rng::new(config.seed);
    let mut global = GlobalMap::new();
    let mut corpus = Corpus::new();
    let mut coverage = Coverage::new();
    let mut dictionary = config.dictionary.clone();

    let initial: Vec<Vec<u8>> = if seeds.is_empty() {
        vec![b"A".to_vec()]
    } else {
        seeds.to_vec()
    };

    // Seed the corpus, recording what each seed covers.
    for seed in &initial {
        coverage.reset();
        let _ = target(seed, &mut coverage);
        let snapshot = coverage.snapshot();
        global.merge(&snapshot);
        corpus.add(seed.clone(), snapshot.iter().map(|&(i, _)| i).collect());
    }

    let mut findings: Vec<Finding> = Vec::new();
    let mut seen_signatures: HashMap<u64, usize> = HashMap::new();
    let started = Instant::now();
    let mut executions = 0u64;
    let mut first_crash_at = None;
    let mut stability = Stability::default();
    let mut trimmed_bytes = 0usize;

    for iteration in 0..config.max_iterations {
        // Choose a parent. Guided mode schedules over the growing corpus;
        // blind mode only ever sees the original seeds.
        let mut candidate = match config.mode {
            Mode::Guided => {
                let index = match config.schedule {
                    Schedule::Weighted => corpus.pick(&mut rng),
                    Schedule::RoundRobin => corpus.pick_round_robin(),
                }
                .unwrap_or(0);
                corpus.input(index).to_vec()
            }
            Mode::Blind => initial[rng.below(initial.len())].clone(),
        };
        let donor = match config.mode {
            Mode::Guided => corpus.random_input(&mut rng).to_vec(),
            Mode::Blind => initial[rng.below(initial.len())].clone(),
        };

        if config.grammar.is_some() && rng.one_in(8) {
            candidate = generate(config.grammar.expect("grammar exists"), &mut rng);
        } else {
            havoc(&mut candidate, &donor, &mut rng);
        }
        if !dictionary.is_empty() && rng.one_in(4) {
            if rng.one_in(3) {
                mutate_token(&mut candidate, &dictionary, &mut rng);
            } else if rng.one_in(2) {
                dictionary.overwrite(&mut candidate, &mut rng);
            } else {
                dictionary.insert(&mut candidate, &mut rng);
            }
        }
        if let Some(fixup) = config.fixup {
            fixup(&mut candidate);
        }

        coverage.reset();
        let outcome = target(&candidate, &mut coverage);
        executions += 1;
        let snapshot = coverage.snapshot();

        if config.mode == Mode::Guided && global.would_be_interesting(&snapshot) {
            stability.checks += 1;
            let mut stable = true;
            for _ in 1..config.stability_runs.max(1) {
                coverage.reset();
                let _ = target(&candidate, &mut coverage);
                executions += 1;
                if coverage.snapshot() != snapshot {
                    stable = false;
                    break;
                }
            }
            if stable {
                let mut admitted = candidate.clone();
                if config.trim_corpus {
                    let original_len = admitted.len();
                    let mut trim_coverage_map = Coverage::new();
                    admitted = trim_coverage(&admitted, |trial| {
                        let mut trial = trial.to_vec();
                        if let Some(fixup) = config.fixup {
                            fixup(&mut trial);
                        }
                        trim_coverage_map.reset();
                        let _ = target(&trial, &mut trim_coverage_map);
                        executions += 1;
                        trim_coverage_map.snapshot() == snapshot
                    });
                    if let Some(fixup) = config.fixup {
                        fixup(&mut admitted);
                    }
                    trimmed_bytes += original_len - admitted.len();
                }
                if config.auto_dictionary {
                    for token in harvest_tokens(&admitted) {
                        dictionary.learn(token);
                    }
                }
                global.merge(&snapshot);
                corpus.add(
                    admitted,
                    snapshot.iter().map(|&(index, _)| index).collect(),
                );
            } else {
                stability.unstable_inputs += 1;
            }
        }

        if let Outcome::Crash(message) = outcome {
            let sig = signature(&snapshot);
            if first_crash_at.is_none() {
                first_crash_at = Some(iteration);
            }
            // Deduplicate by coverage signature, not by bytes: one bug reached
            // ten ways is one bug, and reporting it ten times is how fuzzers
            // drown the person who has to fix them.
            if !seen_signatures.contains_key(&sig) {
                seen_signatures.insert(sig, findings.len());
                findings.push(Finding {
                    input: candidate,
                    message,
                    iteration,
                    signature: sig,
                });
                if config.stop_on_crash {
                    break;
                }
            }
        }

        if config.progress_interval > 0
            && executions % config.progress_interval == 0
        {
            on_progress(&Progress {
                executions,
                corpus_size: corpus.len(),
                favoured: corpus.favoured_count(),
                edges_found: global.edges_found(),
                unique_bugs: seen_signatures.len(),
                elapsed_secs: started.elapsed().as_secs_f64(),
            });
        }
    }

    Report {
        executions,
        elapsed_secs: started.elapsed().as_secs_f64(),
        findings,
        seed: config.seed,
        edges_found: global.edges_found(),
        corpus_size: corpus.len(),
        favoured: corpus.favoured_count(),
        unique_bugs: seen_signatures.len(),
        first_crash_at,
        stability,
        trimmed_bytes,
    }
}

/// Shrink a crashing input while it still crashes.
///
/// A 4 KiB reproducer says almost nothing; a 6-byte one says exactly which
/// bytes matter. Greedy: try deleting a chunk, keep the deletion whenever the
/// crash survives, then halve the chunk size and repeat.
pub fn minimise(target: TargetFn, input: &[u8]) -> Vec<u8> {
    let mut cov = Coverage::new();
    let crashes = |bytes: &[u8], cov: &mut Coverage| {
        cov.reset();
        matches!(target(bytes, cov), Outcome::Crash(_))
    };

    if !crashes(input, &mut cov) {
        return input.to_vec();
    }

    let mut best = input.to_vec();
    let mut chunk = best.len() / 2;

    while chunk > 0 {
        let mut position = 0;
        while position < best.len() && best.len() > 1 {
            let end = (position + chunk).min(best.len());
            let mut trial = best.clone();
            trial.drain(position..end);
            if !trial.is_empty() && crashes(&trial, &mut cov) {
                best = trial;
            } else {
                position += chunk;
            }
        }
        chunk /= 2;
    }
    best
}

/// Render bytes for a report: printable ASCII as-is, everything else escaped.
pub fn render(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &b in bytes {
        if (0x20..0x7f).contains(&b) && b != b'\\' {
            out.push(b as char);
        } else {
            out.push_str(&format!("\\x{b:02x}"));
        }
    }
    out
}
