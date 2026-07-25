//! nutfuzz CLI.
//!
//!   nutfuzz                              fuzz the buggy target (guided)
//!   nutfuzz --target parse_nested        the four-gate target
//!   nutfuzz --mode blind                 no coverage feedback (the baseline)
//!   nutfuzz --seed N --iterations M      a reproducible campaign
//!   nutfuzz --dict FUZZ,REC              seed the dictionary with tokens
//!   nutfuzz --bench                      blind vs guided, several seeds
//!   nutfuzz --list

use std::process::ExitCode;
use std::time::Duration;

use nutfuzz::exec::ExternalTarget;
use nutfuzz::fuzzer::{fuzz, minimise, render, Config, Mode};
use nutfuzz::{
    fuzz_external, load_corpus, target, write_artifacts, Dictionary, ExternalConfig,
};

fn main() -> ExitCode {
    let mut config = Config::default();
    let mut target_name = String::from("parse_record");
    let mut external_program = None;
    let mut external_args = Vec::new();
    let mut corpus_path = None;
    let mut artifact_path = String::from("crashes");
    let mut timeout_ms = 1_000;
    let mut iterations_set = false;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--list" => {
                println!("targets: {}", target::NAMES.join(", "));
                return ExitCode::SUCCESS;
            }
            "--bench" => return benchmark(),
            "--target" => {
                i += 1;
                match args.get(i) {
                    Some(name) => target_name = name.clone(),
                    None => return fail("--target needs a name"),
                }
            }
            "--external" => {
                i += 1;
                match args.get(i) {
                    Some(program) => external_program = Some(program.clone()),
                    None => return fail("--external needs a program"),
                }
            }
            "--arg" => {
                i += 1;
                match args.get(i) {
                    Some(argument) => external_args.push(argument.clone()),
                    None => return fail("--arg needs a value"),
                }
            }
            "--corpus" => {
                i += 1;
                match args.get(i) {
                    Some(path) => corpus_path = Some(path.clone()),
                    None => return fail("--corpus needs a directory"),
                }
            }
            "--artifacts" => {
                i += 1;
                match args.get(i) {
                    Some(path) => artifact_path = path.clone(),
                    None => return fail("--artifacts needs a directory"),
                }
            }
            "--timeout-ms" => {
                i += 1;
                match args.get(i).and_then(|value| value.parse::<u64>().ok()) {
                    Some(value) if value > 0 => timeout_ms = value,
                    _ => return fail("--timeout-ms needs a positive number"),
                }
            }
            "--mode" => {
                i += 1;
                match args.get(i).map(String::as_str) {
                    Some("blind") => config.mode = Mode::Blind,
                    Some("guided") => config.mode = Mode::Guided,
                    _ => return fail("--mode needs 'blind' or 'guided'"),
                }
            }
            "--seed" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(seed) => config.seed = seed,
                    None => return fail("--seed needs a number"),
                }
            }
            "--iterations" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse().ok()) {
                    Some(n) => {
                        config.max_iterations = n;
                        iterations_set = true;
                    }
                    None => return fail("--iterations needs a number"),
                }
            }
            "--dict" => {
                i += 1;
                match args.get(i) {
                    Some(list) => {
                        config.dictionary =
                            Dictionary::from_tokens(list.split(',').map(|t| t.as_bytes().to_vec()));
                    }
                    None => return fail("--dict needs a comma-separated token list"),
                }
            }
            "--keep-going" => config.stop_on_crash = false,
            other => return fail(&format!("unknown argument '{other}'")),
        }
        i += 1;
    }

    if let Some(program) = external_program {
        return run_external(
            program,
            external_args,
            corpus_path,
            artifact_path,
            timeout_ms,
            iterations_set,
            config,
        );
    }

    let Some(target_fn) = target::by_name(&target_name) else {
        return fail(&format!(
            "unknown target '{target_name}' (try: {})",
            target::NAMES.join(", ")
        ));
    };

    println!("nutfuzz — coverage-guided fuzzing\n");
    println!("  target     {target_name}");
    println!("  mode       {}", match config.mode {
        Mode::Guided => "guided (coverage feedback)",
        Mode::Blind => "blind (no feedback)",
    });
    println!("  seed       {}", config.seed);
    println!("  iterations {}", config.max_iterations);
    if !config.dictionary.is_empty() {
        println!("  dictionary {} token(s)", config.dictionary.len());
    }
    println!();

    let seeds = default_seeds(&target_name);
    let report = fuzz(target_fn, &seeds, &config);

    println!(
        "  {} executions in {:.2}s  ({:.0} exec/s)",
        report.executions,
        report.elapsed_secs,
        report.execs_per_second()
    );
    println!(
        "  edges {}   corpus {} ({} favoured)   unique bugs {}",
        report.edges_found, report.corpus_size, report.favoured, report.unique_bugs
    );

    if report.findings.is_empty() {
        println!("\n  no crashes found");
        return ExitCode::SUCCESS;
    }

    println!("\n  {} unique bug(s):\n", report.findings.len());
    for finding in &report.findings {
        let small = minimise(target_fn, &finding.input);
        println!("    iteration {}", finding.iteration);
        println!("    reason    {}", finding.message);
        println!("    signature {:016x}", finding.signature);
        println!("    input     {} bytes  \"{}\"", finding.input.len(), render(&finding.input));
        println!("    minimised {} bytes  \"{}\"", small.len(), render(&small));
        println!(
            "    replay    nutfuzz --target {} --seed {} --iterations {}\n",
            target_name,
            report.seed,
            finding.iteration + 1
        );
    }
    ExitCode::SUCCESS
}

fn run_external(
    program: String,
    args: Vec<String>,
    corpus_path: Option<String>,
    artifact_path: String,
    timeout_ms: u64,
    iterations_set: bool,
    config: Config,
) -> ExitCode {
    let seeds = match corpus_path {
        Some(path) => match load_corpus(&path) {
            Ok(seeds) if !seeds.is_empty() => seeds,
            Ok(_) => return fail("the corpus directory contains no files"),
            Err(error) => return fail(&format!("read corpus: {error}")),
        },
        None => vec![b"A".to_vec()],
    };
    let max_iterations = if iterations_set {
        config.max_iterations
    } else {
        1_000
    };
    let target = ExternalTarget::new(&program)
        .with_args(args)
        .with_timeout(Duration::from_millis(timeout_ms));
    let external_config = ExternalConfig {
        seed: config.seed,
        max_iterations,
        stop_on_finding: config.stop_on_crash,
        dictionary: config.dictionary,
    };

    println!("nutfuzz — deterministic external campaign\n");
    println!("  program    {program}");
    println!("  seed       {}", external_config.seed);
    println!("  iterations {}", external_config.max_iterations);
    println!("  timeout    {timeout_ms} ms");
    println!("  corpus     {} seed(s)\n", seeds.len());

    let report = fuzz_external(&target, &seeds, &external_config);
    if let Some(error) = &report.harness_error {
        return fail(&format!("external harness: {error}"));
    }
    println!("  {} executions", report.executions);
    if report.findings.is_empty() {
        println!("\n  no crashes or hangs found");
        return ExitCode::SUCCESS;
    }

    println!("\n  {} unique finding(s):\n", report.findings.len());
    for finding in &report.findings {
        println!("    iteration {}", finding.iteration);
        println!("    result    {:?}", finding.result);
        println!("    input     {} bytes", finding.input.len());
        println!(
            "    minimised {} bytes  \"{}\"\n",
            finding.minimised.len(),
            render(&finding.minimised)
        );
    }
    match write_artifacts(&artifact_path, &report) {
        Ok(paths) => println!("  wrote {} artifact file(s) to {artifact_path}", paths.len()),
        Err(error) => return fail(&format!("write artifacts: {error}")),
    }
    ExitCode::SUCCESS
}

fn default_seeds(target_name: &str) -> Vec<Vec<u8>> {
    if target_name.starts_with("parse_nested") {
        vec![b"AAAAAAAA".to_vec()]
    } else {
        vec![b"REC".to_vec(), b"REC\x01\x00".to_vec()]
    }
}

/// The measurement that justifies coverage feedback: executions to the first
/// crash, blind versus guided, on a bug behind four gates.
fn benchmark() -> ExitCode {
    const BUDGET: u64 = 300_000;
    let seeds_list = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    println!("nutfuzz benchmark — executions to first crash on `parse_nested`");
    println!("(a bug behind four gates: magic, version, mode, size)\n");
    println!("  budget {BUDGET} executions, {} seeds\n", seeds_list.len());

    let run = |mode: Mode, seed: u64| {
        fuzz(
            target::parse_nested,
            &[b"AAAAAAAA".to_vec()],
            &Config {
                seed,
                max_iterations: BUDGET,
                stop_on_crash: true,
                mode,
                ..Config::default()
            },
        )
    };

    let mut guided_hits = 0;
    let mut guided_execs: Vec<u64> = Vec::new();
    let mut blind_hits = 0;
    let mut blind_execs: Vec<u64> = Vec::new();

    for &seed in &seeds_list {
        let g = run(Mode::Guided, seed);
        if let Some(at) = g.first_crash_at {
            guided_hits += 1;
            guided_execs.push(at);
        }
        let b = run(Mode::Blind, seed);
        if let Some(at) = b.first_crash_at {
            blind_hits += 1;
            blind_execs.push(at);
        }
    }

    let median = |mut v: Vec<u64>| -> String {
        if v.is_empty() {
            return "not found".to_string();
        }
        v.sort_unstable();
        v[v.len() / 2].to_string()
    };

    println!("  {:<28} {:>10} {:>18}", "fuzzer", "found", "median execs");
    println!(
        "  {:<28} {:>7}/{:<2} {:>18}",
        "blind mutation",
        blind_hits,
        seeds_list.len(),
        median(blind_execs)
    );
    println!(
        "  {:<28} {:>7}/{:<2} {:>18}",
        "coverage-guided",
        guided_hits,
        seeds_list.len(),
        median(guided_execs)
    );

    let sample = run(Mode::Guided, 1);
    println!(
        "\n  a guided run grew the corpus to {} entries ({} favoured) and found {} edges",
        sample.corpus_size, sample.favoured, sample.edges_found
    );
    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    eprintln!("nutfuzz: {message}");
    eprintln!(
        "usage: nutfuzz [--target NAME] [--mode blind|guided] [--seed N]\n\
         \x20              [--iterations N] [--dict a,b,c] [--keep-going]\n\
         \x20              [--external PROGRAM] [--arg VALUE] [--corpus DIR]\n\
         \x20              [--timeout-ms N] [--artifacts DIR] [--bench] [--list]"
    );
    ExitCode::FAILURE
}
