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

use nutfuzz::fuzzer::{fuzz, minimise, render, Config, Mode};
use nutfuzz::{target, Dictionary};

fn main() -> ExitCode {
    let mut config = Config::default();
    let mut target_name = String::from("parse_record");

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
                    Some(n) => config.max_iterations = n,
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
         \x20              [--bench] [--list]"
    );
    ExitCode::FAILURE
}
