//! nutfuzz CLI.
//!
//!   cargo run --release                          fuzz the buggy target
//!   cargo run --release -- --target parse_record_fixed
//!   cargo run --release -- --seed 12345 --iterations 500000
//!   cargo run --release -- --list

use std::process::ExitCode;

use nutfuzz::fuzzer::{fuzz, minimise, render, Config};
use nutfuzz::target;

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
            "--target" => {
                i += 1;
                match args.get(i) {
                    Some(name) => target_name = name.clone(),
                    None => return fail("--target needs a name"),
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

    println!("nutfuzz milestone 0 — deterministic mutation fuzzing\n");
    println!("  target     {target_name}");
    println!("  seed       {}", config.seed);
    println!("  iterations {}\n", config.max_iterations);

    let seeds = vec![b"REC".to_vec(), b"REC\x01\x00".to_vec()];
    let report = fuzz(target_fn, &seeds, &config);

    println!(
        "  {} executions in {:.2}s  ({:.0} exec/s)",
        report.executions,
        report.elapsed_secs,
        report.execs_per_second()
    );

    if report.findings.is_empty() {
        println!("\n  no crashes found");
        return ExitCode::SUCCESS;
    }

    println!("\n  {} crash(es) found:\n", report.findings.len());
    for finding in &report.findings {
        let small = minimise(target_fn, &finding.input);
        println!("    iteration {}", finding.iteration);
        println!("    reason    {}", finding.message);
        println!(
            "    input     {} bytes  \"{}\"",
            finding.input.len(),
            render(&finding.input)
        );
        println!(
            "    minimised {} bytes  \"{}\"",
            small.len(),
            render(&small)
        );
        println!(
            "    replay    cargo run --release -- --seed {} --iterations {}\n",
            report.seed,
            finding.iteration + 1
        );
    }

    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    eprintln!("nutfuzz: {message}");
    eprintln!(
        "usage: nutfuzz [--target NAME] [--seed N] [--iterations N] \
         [--keep-going] [--list]"
    );
    ExitCode::FAILURE
}
