//! nutfuzz — a coverage-guided fuzzer built from the mutation engine up.
//!
//! Milestone 0 is a deterministic mutation fuzzer: a seeded PRNG, an AFL-style
//! mutation engine, a fuzzing loop, and a crash minimiser. It finds a planted
//! length-field bug in a toy parser and shrinks the reproducer to the bytes
//! that matter.
//!
//! ```
//! use nutfuzz::{fuzzer::{fuzz, Config}, target};
//!
//! let report = fuzz(target::parse_record, &[b"REC".to_vec()], &Config::default());
//! assert!(!report.findings.is_empty(), "the planted bug should be found");
//! ```

pub mod fuzzer;
pub mod mutate;
pub mod rng;
pub mod target;

pub use fuzzer::{fuzz, minimise, Config, Finding, Report};
pub use rng::Rng;
pub use target::{Outcome, TargetFn};
