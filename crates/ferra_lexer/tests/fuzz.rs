use ferra_lexer::Lexer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn lexer_never_panics_on_random_input(s in ".{0,512}") {
        let _ = Lexer::new(&s).lex();
    }
}
