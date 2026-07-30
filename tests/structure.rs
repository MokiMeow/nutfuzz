use nutfuzz::structure::{generate, harvest_tokens, mutate_token, Grammar};
use nutfuzz::{Dictionary, Rng};

#[test]
fn harvests_unique_printable_runs_in_the_supported_size_range() {
    let tokens = harvest_tokens(b"\0alpha\0beta_123\0abc\0alpha\0");
    assert_eq!(tokens, vec![b"alpha".to_vec(), b"beta_123".to_vec()]);
}

#[test]
fn token_mutation_preserves_delimiters_around_the_replaced_token() {
    let dictionary = Dictionary::from_tokens(["false"]);
    let mut input = br#"{"enabled":true}"#.to_vec();
    mutate_token(&mut input, &dictionary, &mut Rng::new(3));
    assert_eq!(input, br#"{"enabled":false}"#);
}

#[test]
fn json_grammar_always_emits_balanced_objects() {
    let mut rng = Rng::new(7);
    for _ in 0..1_000 {
        let input = generate(Grammar::Json, &mut rng);
        assert_eq!(input.first(), Some(&b'{'));
        assert_eq!(input.last(), Some(&b'}'));
        assert_eq!(
            input.iter().filter(|&&byte| byte == b'{').count(),
            input.iter().filter(|&&byte| byte == b'}').count()
        );
        assert_eq!(
            input.iter().filter(|&&byte| byte == b'"').count() % 2,
            0
        );
    }
}
