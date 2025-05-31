use ferra_lexer::Lexer;
use proptest::prelude::*;

proptest! {
    #[test]
    fn lexer_never_panics_on_random_input(s in ".{0,512}") {
        let _ = Lexer::new(&s).lex();
    }
    
    #[test]
    fn lexer_handles_raw_string_fuzzing(
        hash_count in 0usize..=5,
        content in "[^\"]*", // Content without quotes to avoid early termination
        malformed in prop::bool::ANY
    ) {
        // Generate raw string with variable hash count
        let hashes = "#".repeat(hash_count);
        let input = if malformed {
            // Sometimes create malformed strings for error path testing
            format!("r{}\"{}\"", hashes, content)
        } else {
            format!("r{}\"{}\"{}\"", hashes, content, hashes)
        };
        
        // Should never panic, regardless of input
        let _ = Lexer::new(&input).lex();
    }
    
    #[test]
    fn lexer_handles_multiline_string_fuzzing(
        content in "[^\"]{0,100}",
        has_newlines in prop::bool::ANY,
        malformed in prop::bool::ANY
    ) {
        let inner_content = if has_newlines {
            content.replace("n", "\n") // Add some newlines
        } else {
            content
        };
        
        let input = if malformed {
            format!("\"\"\"{}\"", inner_content) // Missing closing quotes
        } else {
            format!("\"\"\"{}\"\"\"", inner_content)
        };
        
        // Should never panic
        let _ = Lexer::new(&input).lex();
    }
}
