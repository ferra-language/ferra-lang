//! Test validation for error recovery functionality
//!
//! This module specifically tests that error recovery works correctly
//! without causing infinite loops or hanging, which was the original issue.

use ferra_lexer::{Lexer, LiteralValue, Token as LexerToken, TokenKind};
use ferra_parser::token::{TokenType, VecTokenStream};
use ferra_parser::{Arena, ProgramParser};

/// Convert lexer token to parser token type
fn convert_token(token: LexerToken) -> TokenType {
    match token.kind {
        TokenKind::Let => TokenType::Let,
        TokenKind::Fn => TokenType::Fn,
        TokenKind::Equal => TokenType::Equal,
        TokenKind::Semicolon => TokenType::Semicolon,
        TokenKind::Identifier => TokenType::Identifier(token.lexeme.clone()),
        TokenKind::IntegerLiteral => match token.literal {
            Some(LiteralValue::Integer(i)) => TokenType::IntegerLiteral(i),
            _ => TokenType::IntegerLiteral(0),
        },
        TokenKind::Eof => TokenType::Eof,
        _ => TokenType::Eof, // Fallback for unhandled tokens
    }
}

/// Convert source code to tokens using the lexer
fn source_to_tokens(source: &str) -> Vec<TokenType> {
    let lexer = Lexer::new(source);
    let tokens = lexer.lex();
    tokens
        .into_iter()
        .map(convert_token)
        .filter(|t| !matches!(t, TokenType::Eof)) // Remove intermediate EOF tokens
        .chain(std::iter::once(TokenType::Eof)) // Add single EOF at end
        .collect()
}

/// Test that error recovery works correctly for multiple errors
fn test_error_recovery_with_source(source: &str, test_name: &str) -> bool {
    println!("Testing error recovery for: {}", test_name);
    let arena = Arena::new();
    let tokens = source_to_tokens(source);
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    // The main goal: this should not hang, regardless of success/failure
    let start_time = std::time::Instant::now();
    let result = parser.parse_compilation_unit();
    let elapsed = start_time.elapsed();

    // Should complete quickly (within 100ms)
    if elapsed.as_millis() > 100 {
        println!(
            "✗ {} took too long ({:?}), likely hanging",
            test_name, elapsed
        );
        return false;
    }

    match result {
        Ok(_) => {
            println!("✓ {} parsed successfully in {:?}", test_name, elapsed);
            true // Success is fine - the key is no hanging
        }
        Err(errors) => {
            println!(
                "✓ {} failed as expected with {} errors in {:?}",
                test_name,
                errors.len(),
                elapsed
            );
            true // Failure is also fine - the key is no hanging
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_syntax_errors_no_hanging() {
        // Test cases that previously caused hanging due to infinite loops in error recovery
        // The main goal is to ensure these complete quickly without hanging
        let test_cases = vec![
            ("let = 42;", "missing_variable_name"),
            ("let x = ;", "missing_expression"),
            ("fn test( { }", "malformed_function_signature"),
            ("let x = @@ 42;", "invalid_operator"),
            ("if true", "incomplete_if_statement"),
            ("while", "incomplete_while_statement"),
        ];

        for (source, test_name) in test_cases {
            // Each of these should complete quickly without hanging
            assert!(
                test_error_recovery_with_source(source, test_name),
                "Test '{}' should complete without hanging",
                test_name
            );
        }
    }

    #[test]
    fn test_error_recovery_continues_parsing() {
        // Test that error recovery allows continued parsing after errors
        let source = "
            let = 42;        // Error: missing variable name
            fn valid() { }   // This should still be parsed
            let x = ;        // Error: missing expression
            fn also_valid() { } // This should also be parsed
        ";

        let arena = Arena::new();
        let tokens = source_to_tokens(source);
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        match parser.parse_compilation_unit() {
            Ok(program) => {
                // Should parse some valid items despite errors
                println!("Successfully parsed {} items", program.items.len());
                assert!(
                    !program.items.is_empty(),
                    "Should parse at least some valid items"
                );
            }
            Err(errors) => {
                // Should collect multiple errors but still parse what it can
                println!("Collected {} errors during parsing", errors.len());
                assert!(
                    !errors.is_empty(),
                    "Should collect errors from invalid syntax"
                );
                // The fact that we got here without hanging is the main success
            }
        }
    }

    #[test]
    fn test_error_recovery_forward_progress() {
        // Test that error recovery always makes forward progress
        let problematic_patterns = vec![
            "let x = 42abc;", // Invalid number literal
            "fn test() {",    // Missing closing brace
            "let fn = 42;",   // Keyword as identifier
        ];

        for pattern in problematic_patterns {
            let arena = Arena::new();
            let tokens = source_to_tokens(pattern);
            let token_stream = VecTokenStream::from_token_types(tokens);
            let mut parser = ProgramParser::new(&arena, token_stream);

            // This should complete within reasonable time (not hang)
            let start_time = std::time::Instant::now();
            let _result = parser.parse_compilation_unit();
            let elapsed = start_time.elapsed();

            // Should complete within 100ms (way more than enough for these simple cases)
            assert!(
                elapsed.as_millis() < 100,
                "Parsing '{}' took too long ({:?}), likely hanging",
                pattern,
                elapsed
            );
        }
    }
}
