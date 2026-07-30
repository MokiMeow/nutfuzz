//! nutfuzz — a coverage-guided fuzzer built from the mutation engine up.
//!
//! A seeded PRNG, an AFL-style mutation engine, edge-coverage feedback, a
//! scheduled corpus, dictionaries, and a crash minimiser — with no
//! dependencies.
//!
//! ```
//! use nutfuzz::{fuzzer::{fuzz, Config, Mode}, target};
//!
//! let config = Config { mode: Mode::Guided, ..Config::default() };
//! let report = fuzz(target::parse_record, &[b"REC".to_vec()], &config);
//! assert!(!report.findings.is_empty(), "the planted bug should be found");
//! ```

pub mod corpus;
pub mod coverage;
pub mod dictionary;
pub mod exec;
pub mod external;
pub mod fuzzer;
pub mod mutate;
pub mod rng;
pub mod target;
pub mod trim;

pub use corpus::Corpus;
pub use coverage::{Coverage, GlobalMap};
pub use dictionary::Dictionary;
pub use fuzzer::{fuzz, minimise, Config, Finding, Mode, Report};
pub use external::{
    fuzz_external, load_corpus, minimise_external, write_artifacts, ExternalConfig,
    ExternalFinding, ExternalReport,
};
pub use rng::Rng;
pub use target::{Outcome, TargetFn};
