//! Deterministic campaigns for external binaries.
//!
//! External targets do not expose coverage yet, so this loop is deliberately
//! blind. It still provides the parts a real campaign needs: seeded mutation,
//! correct crash/hang/harness classification, result-class deduplication,
//! minimisation, sorted corpus loading, and reproducible artifacts.

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::dictionary::Dictionary;
use crate::exec::{Executor, ExternalTarget, RunResult};
use crate::fuzzer::render;
use crate::mutate::havoc;
use crate::rng::Rng;

pub struct ExternalConfig {
    pub seed: u64,
    pub max_iterations: u64,
    pub stop_on_finding: bool,
    pub dictionary: Dictionary,
}

impl Default for ExternalConfig {
    fn default() -> Self {
        Self {
            seed: 0x5eed,
            max_iterations: 10_000,
            stop_on_finding: true,
            dictionary: Dictionary::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalFinding {
    pub input: Vec<u8>,
    pub minimised: Vec<u8>,
    pub result: RunResult,
    pub iteration: u64,
}

#[derive(Debug)]
pub struct ExternalReport {
    pub executions: u64,
    pub seed: u64,
    pub findings: Vec<ExternalFinding>,
    pub harness_error: Option<String>,
}

pub fn fuzz_external(
    target: &ExternalTarget,
    seeds: &[Vec<u8>],
    config: &ExternalConfig,
) -> ExternalReport {
    let mut target = target;
    fuzz_executor(&mut target, seeds, config)
}

pub fn fuzz_executor(
    target: &mut dyn Executor,
    seeds: &[Vec<u8>],
    config: &ExternalConfig,
) -> ExternalReport {
    let corpus = if seeds.is_empty() {
        vec![b"A".to_vec()]
    } else {
        seeds.to_vec()
    };
    let mut random = Rng::new(config.seed);
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    let mut executions = 0;
    let mut harness_error = None;

    for iteration in 0..config.max_iterations {
        let mut candidate = corpus[random.below(corpus.len())].clone();
        let donor = &corpus[random.below(corpus.len())];
        havoc(&mut candidate, donor, &mut random);
        if !config.dictionary.is_empty() && random.one_in(4) {
            if random.one_in(2) {
                config.dictionary.overwrite(&mut candidate, &mut random);
            } else {
                config.dictionary.insert(&mut candidate, &mut random);
            }
        }

        let result = target.run(&candidate);
        executions += 1;
        match &result {
            RunResult::HarnessError(error) => {
                harness_error = Some(error.clone());
                break;
            }
            RunResult::Exited(_) => {}
            RunResult::Crashed(..) | RunResult::Hang => {
                let Some(key) = finding_key(&result) else {
                    continue;
                };
                if seen.insert(key) {
                    let minimised = minimise_executor(target, &candidate, &result);
                    findings.push(ExternalFinding {
                        input: candidate,
                        minimised,
                        result,
                        iteration,
                    });
                    if config.stop_on_finding {
                        break;
                    }
                }
            }
        }
    }

    ExternalReport {
        executions,
        seed: config.seed,
        findings,
        harness_error,
    }
}

pub fn minimise_external(
    target: &ExternalTarget,
    input: &[u8],
    expected: &RunResult,
) -> Vec<u8> {
    let mut target = target;
    minimise_executor(&mut target, input, expected)
}

pub fn minimise_executor(
    target: &mut dyn Executor,
    input: &[u8],
    expected: &RunResult,
) -> Vec<u8> {
    let Some(expected_key) = finding_key(expected) else {
        return input.to_vec();
    };
    let mut preserves = |candidate: &[u8]| {
        finding_key(&target.run(candidate))
            .map(|key| key == expected_key)
            .unwrap_or(false)
    };
    if !preserves(input) {
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
            if !trial.is_empty() && preserves(&trial) {
                best = trial;
            } else {
                position += chunk;
            }
        }
        chunk /= 2;
    }
    best
}

pub fn load_corpus(path: impl AsRef<Path>) -> io::Result<Vec<Vec<u8>>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    paths.into_iter().map(fs::read).collect()
}

pub fn write_artifacts(
    path: impl AsRef<Path>,
    report: &ExternalReport,
) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(path.as_ref())?;
    let mut written = Vec::new();
    for finding in &report.findings {
        let key = finding_key(&finding.result).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "external artifact is not a crash or hang",
            )
        })?;
        let id = artifact_id(&key, &finding.minimised);
        let stem = format!("finding-{id:016x}");
        let input_path = path.as_ref().join(format!("{stem}.bin"));
        let metadata_path = path.as_ref().join(format!("{stem}.txt"));
        write_private(&input_path, &finding.minimised)?;
        write_private(
            &metadata_path,
            format!(
                "seed: {}\niteration: {}\nresult: {:?}\noriginal-bytes: {}\n\
                 minimised-bytes: {}\nrendered: {}\n",
                report.seed,
                finding.iteration,
                finding.result,
                finding.input.len(),
                finding.minimised.len(),
                render(&finding.minimised)
            )
            .as_bytes(),
        )?;
        written.push(input_path);
        written.push(metadata_path);
    }
    Ok(written)
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)
}

fn finding_key(result: &RunResult) -> Option<String> {
    match result {
        RunResult::Crashed(signal, name) => Some(format!("crash:{signal}:{name}")),
        RunResult::Hang => Some("hang".to_owned()),
        RunResult::Exited(_) | RunResult::HarnessError(_) => None,
    }
}

fn artifact_id(key: &str, input: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in key.as_bytes().iter().chain(input) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

#[cfg(all(test, unix))]
mod tests {
    use super::{fuzz_external, load_corpus, write_artifacts, ExternalConfig};
    use crate::exec::ExternalTarget;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn crash_target() -> ExternalTarget {
        ExternalTarget::new("/bin/sh").with_args(vec!["-c".into(), "kill -SEGV $$".into()])
    }

    #[test]
    fn campaign_is_reproducible_and_minimises_a_real_signal() {
        let config = ExternalConfig {
            seed: 42,
            max_iterations: 10,
            ..ExternalConfig::default()
        };
        let a = fuzz_external(&crash_target(), &[b"seed-input".to_vec()], &config);
        let b = fuzz_external(&crash_target(), &[b"seed-input".to_vec()], &config);
        assert_eq!(a.findings, b.findings);
        assert_eq!(a.findings.len(), 1);
        assert_eq!(a.findings[0].minimised.len(), 1);
    }

    #[test]
    fn clean_and_broken_harnesses_are_not_findings() {
        let config = ExternalConfig {
            seed: 7,
            max_iterations: 20,
            ..ExternalConfig::default()
        };
        let clean = fuzz_external(
            &ExternalTarget::new("/bin/true"),
            &[b"seed".to_vec()],
            &config,
        );
        assert!(clean.findings.is_empty());
        assert!(clean.harness_error.is_none());

        let broken = fuzz_external(
            &ExternalTarget::new("/not/a/real/program"),
            &[b"seed".to_vec()],
            &config,
        );
        assert!(broken.findings.is_empty());
        assert!(broken.harness_error.is_some());
    }

    #[test]
    fn corpus_order_and_artifact_names_are_deterministic() {
        let directory = TempDir::new("artifacts");
        fs::write(directory.0.join("b"), b"second").unwrap();
        fs::write(directory.0.join("a"), b"first").unwrap();
        assert_eq!(
            load_corpus(&directory.0).unwrap(),
            [b"first".to_vec(), b"second".to_vec()]
        );

        let report = fuzz_external(
            &crash_target(),
            &[b"artifact".to_vec()],
            &ExternalConfig::default(),
        );
        let first = write_artifacts(directory.0.join("out"), &report).unwrap();
        let second = write_artifacts(directory.0.join("out"), &report).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 2);
        for path in first {
            let mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "finding artifacts must be private");
        }
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "nutfuzz-{name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
