//! The fuzzing loop: pick an input, mutate it, run the target, record what
//! happened, repeat.
//!
//! Milestone 0 is a *mutation* fuzzer — it has no coverage feedback yet, so the
//! corpus does not grow from interesting inputs. That is deliberate: it
//! establishes the loop, the determinism guarantee, and the crash-reporting
//! path, so milestone 1 can add coverage and **measure** what the feedback
//! actually buys. See docs/decisions/0002-mutation-before-coverage.md.

use std::time::Instant;

use crate::mutate::havoc;
use crate::rng::Rng;
use crate::target::{Outcome, TargetFn};

pub struct Config {
    pub seed: u64,
    pub max_iterations: u64,
    /// Stop as soon as the first crash is found.
    pub stop_on_crash: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 0x5EED,
            max_iterations: 200_000,
            stop_on_crash: true,
        }
    }
}

/// A reproducible crash.
#[derive(Debug, Clone)]
pub struct Finding {
    pub input: Vec<u8>,
    pub message: String,
    /// Which iteration produced it — with the seed, this replays the campaign.
    pub iteration: u64,
}

#[derive(Debug)]
pub struct Report {
    pub executions: u64,
    pub elapsed_secs: f64,
    pub findings: Vec<Finding>,
    pub seed: u64,
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

/// Run a campaign against `target`, starting from `seeds`.
pub fn fuzz(target: TargetFn, seeds: &[Vec<u8>], config: &Config) -> Report {
    let mut rng = Rng::new(config.seed);
    let corpus: Vec<Vec<u8>> = if seeds.is_empty() {
        vec![b"REC".to_vec()]
    } else {
        seeds.to_vec()
    };

    let mut findings: Vec<Finding> = Vec::new();
    let started = Instant::now();
    let mut executions = 0u64;

    for iteration in 0..config.max_iterations {
        // Pick a parent and a splice donor from the corpus.
        let mut candidate = corpus[rng.below(corpus.len())].clone();
        let donor = &corpus[rng.below(corpus.len())];

        havoc(&mut candidate, donor, &mut rng);

        executions += 1;
        if let Outcome::Crash(message) = target(&candidate) {
            // Only record genuinely new crash inputs.
            let seen = findings.iter().any(|f| f.input == candidate);
            if !seen {
                findings.push(Finding {
                    input: candidate,
                    message,
                    iteration,
                });
                if config.stop_on_crash {
                    break;
                }
            }
        }
    }

    Report {
        executions,
        elapsed_secs: started.elapsed().as_secs_f64(),
        findings,
        seed: config.seed,
    }
}

/// Shrink a crashing input while it still crashes.
///
/// A 4 KiB reproducer tells you almost nothing; a 6-byte one tells you exactly
/// which bytes matter. Greedy: repeatedly try deleting a chunk, and keep the
/// deletion whenever the crash survives.
pub fn minimise(target: TargetFn, input: &[u8]) -> Vec<u8> {
    let crashes = |bytes: &[u8]| matches!(target(bytes), Outcome::Crash(_));
    if !crashes(input) {
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

            if !trial.is_empty() && crashes(&trial) {
                best = trial; // the cut was safe; retry at the same position
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
