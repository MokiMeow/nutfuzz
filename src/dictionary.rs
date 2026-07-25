//! Dictionaries: getting past gates that byte-level mutation cannot open.
//!
//! A four-byte magic value like `"FUZZ"` is one in 2^32 by chance — a blind
//! mutator will never type it. With the token in a dictionary it becomes a
//! single mutation. This is why real fuzzers ship format dictionaries, and why
//! a fuzzer without one looks mysteriously bad on structured formats.

use crate::mutate::MAX_LEN;
use crate::rng::Rng;

#[derive(Default, Clone)]
pub struct Dictionary {
    tokens: Vec<Vec<u8>>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tokens<I, T>(tokens: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Vec<u8>>,
    {
        let mut dictionary = Self::new();
        for token in tokens {
            dictionary.learn(token);
        }
        dictionary
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Learn a token seen in the target's input checks.
    ///
    /// Real fuzzers harvest these automatically from comparison instructions
    /// (libFuzzer's value profile). Duplicates are ignored so the dictionary
    /// does not grow without bound.
    pub fn learn(&mut self, token: impl Into<Vec<u8>>) {
        let token = token.into();
        if token.is_empty() || token.len() > 32 {
            return;
        }
        if !self.tokens.contains(&token) {
            self.tokens.push(token);
        }
    }

    /// Overwrite bytes at a random offset with a token.
    pub fn overwrite(&self, input: &mut Vec<u8>, rng: &mut Rng) {
        if self.tokens.is_empty() || input.is_empty() {
            return;
        }
        let token = &self.tokens[rng.below(self.tokens.len())];
        if token.len() > input.len() {
            // Too long to overwrite in place — put it at the front instead.
            if input.len().saturating_add(token.len()) > MAX_LEN {
                return;
            }
            let at = 0;
            input.splice(at..at, token.iter().copied());
            return;
        }
        let at = rng.below(input.len() - token.len() + 1);
        input[at..at + token.len()].copy_from_slice(token);
    }

    /// Insert a token at a random offset without removing anything.
    pub fn insert(&self, input: &mut Vec<u8>, rng: &mut Rng) {
        if self.tokens.is_empty() {
            return;
        }
        let token = &self.tokens[rng.below(self.tokens.len())];
        if input.len().saturating_add(token.len()) > MAX_LEN {
            return;
        }
        let at = rng.below(input.len() + 1);
        input.splice(at..at, token.iter().copied());
    }
}

#[cfg(test)]
mod tests {
    use super::Dictionary;
    use crate::mutate::MAX_LEN;
    use crate::rng::Rng;

    #[test]
    fn overwrite_places_a_token_in_the_input() {
        let dict = Dictionary::from_tokens(["FUZZ"]);
        let mut rng = Rng::new(1);
        let mut found = false;
        for _ in 0..200 {
            let mut input = vec![b'.'; 16];
            dict.overwrite(&mut input, &mut rng);
            if input.windows(4).any(|w| w == b"FUZZ") {
                found = true;
                break;
            }
        }
        assert!(found, "the token should appear in the mutated input");
    }

    #[test]
    fn a_token_longer_than_the_input_still_lands() {
        let dict = Dictionary::from_tokens(["LONGTOKEN"]);
        let mut rng = Rng::new(2);
        let mut input = vec![b'x'; 3];
        dict.overwrite(&mut input, &mut rng);
        assert!(input.windows(9).any(|w| w == b"LONGTOKEN"));
    }

    #[test]
    fn empty_dictionary_is_a_no_op() {
        let dict = Dictionary::new();
        let mut rng = Rng::new(3);
        let mut input = b"unchanged".to_vec();
        dict.overwrite(&mut input, &mut rng);
        dict.insert(&mut input, &mut rng);
        assert_eq!(input, b"unchanged");
    }

    #[test]
    fn learning_ignores_duplicates_and_junk() {
        let mut dict = Dictionary::new();
        dict.learn("FUZZ");
        dict.learn("FUZZ");
        dict.learn("");
        dict.learn(vec![0u8; 64]); // too long
        assert_eq!(dict.len(), 1);
    }

    #[test]
    fn insert_grows_the_input() {
        let dict = Dictionary::from_tokens(["AB"]);
        let mut rng = Rng::new(4);
        let mut input = b"xy".to_vec();
        dict.insert(&mut input, &mut rng);
        assert_eq!(input.len(), 4);
    }

    #[test]
    fn tokens_never_grow_an_input_past_the_cap() {
        let dictionary = Dictionary::from_tokens(["FUZZ"]);
        let mut rng = Rng::new(5);
        let mut input = vec![b'x'; MAX_LEN];
        dictionary.insert(&mut input, &mut rng);
        dictionary.overwrite(&mut input, &mut rng);
        assert_eq!(input.len(), MAX_LEN);
    }

    #[test]
    fn constructor_rejects_empty_duplicate_and_oversized_tokens() {
        let dictionary = Dictionary::from_tokens([
            Vec::new(),
            b"token".to_vec(),
            b"token".to_vec(),
            vec![b'x'; 33],
        ]);
        assert_eq!(dictionary.len(), 1);
    }
}
