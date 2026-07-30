use crate::dictionary::Dictionary;
use crate::mutate::MAX_LEN;
use crate::rng::Rng;

pub type FixupFn = fn(&mut Vec<u8>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grammar {
    Json,
}

pub fn harvest_tokens(input: &[u8]) -> Vec<Vec<u8>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, &byte) in input.iter().enumerate() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            start.get_or_insert(index);
        } else if let Some(run_start) = start.take() {
            learn_run(&mut tokens, &input[run_start..index]);
        }
    }
    if let Some(run_start) = start {
        learn_run(&mut tokens, &input[run_start..]);
    }
    tokens
}

fn learn_run(tokens: &mut Vec<Vec<u8>>, run: &[u8]) {
    if (4..=32).contains(&run.len()) && !tokens.iter().any(|token| token == run) {
        tokens.push(run.to_vec());
    }
}

pub fn generate(grammar: Grammar, rng: &mut Rng) -> Vec<u8> {
    match grammar {
        Grammar::Json => {
            const KEYS: [&str; 4] = ["name", "enabled", "count", "mode"];
            const VALUES: [&str; 5] = ["true", "false", "null", "0", "\"fuzz\""];
            format!(
                "{{\"{}\":{}}}",
                rng.pick(&KEYS),
                rng.pick(&VALUES)
            )
            .into_bytes()
        }
    }
}

pub fn mutate_token(input: &mut Vec<u8>, dictionary: &Dictionary, rng: &mut Rng) {
    if dictionary.is_empty() {
        return;
    }

    let replacement = rng.pick(dictionary.tokens());
    let runs = token_runs(input);
    if runs.is_empty() {
        return;
    }

    let replacement_is_literal =
        matches!(replacement.as_slice(), b"true" | b"false" | b"null");
    let matching: Vec<(usize, usize)> = if replacement_is_literal {
        runs.iter()
            .copied()
            .filter(|&(start, end)| {
                matches!(&input[start..end], b"true" | b"false" | b"null")
            })
            .collect()
    } else {
        Vec::new()
    };
    let choices = if matching.is_empty() { &runs } else { &matching };
    let &(start, end) = rng.pick(choices);

    let new_len = input.len() - (end - start) + replacement.len();
    if new_len <= MAX_LEN {
        input.splice(start..end, replacement.iter().copied());
    }
}

fn token_runs(input: &[u8]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (index, &byte) in input.iter().enumerate() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            start.get_or_insert(index);
        } else if let Some(run_start) = start.take() {
            runs.push((run_start, index));
        }
    }
    if let Some(run_start) = start {
        runs.push((run_start, input.len()));
    }
    runs
}
