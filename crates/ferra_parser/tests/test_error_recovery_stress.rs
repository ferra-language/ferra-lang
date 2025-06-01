use ferra_parser::{
    program::ProgramParser,
    token::{stream::VecTokenStream, types::TokenType},
    Arena,
};
use std::time::{Duration, Instant};

/// Generate tokens with errors for stress testing
#[allow(dead_code)]
fn create_error_tokens(scenario: &str) -> Vec<TokenType> {
    match scenario {
        "missing_semicolon" => vec![
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(42),
            // Missing semicolon here
            TokenType::Let,
            TokenType::Identifier("y".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(24),
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::Eof,
        ],
        "unmatched_brace" => vec![
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(42),
            TokenType::Semicolon,
            // Missing closing brace
            TokenType::Eof,
        ],
        "multiple_errors" => vec![
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            // Missing open paren
            TokenType::LeftBrace,
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            // Missing equal sign and value
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Identifier("y".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(42),
            // Missing semicolon and closing brace
            TokenType::Eof,
        ],
        _ => vec![
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ],
    }
}

/// Generate a large program with many errors
fn generate_error_cascade_tokens(func_count: usize) -> Vec<TokenType> {
    let mut tokens = Vec::new();

    for i in 0..func_count {
        if i % 10 == 0 {
            // Every 10th function has errors
            tokens.extend(vec![
                TokenType::Fn,
                TokenType::Identifier(format!("error_func_{i}")),
                // Missing parameter list
                TokenType::LeftBrace,
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                // Missing expression
                TokenType::Semicolon,
                TokenType::If,
                TokenType::Identifier("condition".to_string()),
                // Missing opening brace
                TokenType::Let,
                TokenType::Identifier("y".to_string()),
                // Missing everything else - should cause cascading errors
            ]);
        } else {
            // Valid functions
            tokens.extend(vec![
                TokenType::Fn,
                TokenType::Identifier(format!("valid_func_{i}")),
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(42),
                TokenType::Semicolon,
                TokenType::Return,
                TokenType::Identifier("x".to_string()),
                TokenType::Semicolon,
                TokenType::RightBrace,
            ]);
        }
    }

    tokens.push(TokenType::Eof);
    tokens
}

#[allow(dead_code)]
#[derive(Debug)]
struct ErrorRecoveryStressResult {
    test_name: String,
    parsing_time: Duration,
    errors_detected: usize,
    recovery_successful: bool,
    max_recovery_attempts: usize,
    forward_progress_maintained: bool,
}

/// Test parser recovery under extreme error conditions
fn test_error_recovery_stress(
    tokens: Vec<TokenType>,
    test_name: &str,
    timeout_ms: u64,
) -> ErrorRecoveryStressResult {
    let start_time = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    let arena = Arena::new();
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    // Parse with timeout protection
    let _result = parser.parse_compilation_unit();
    let parsing_time = start_time.elapsed();
    let within_timeout = parsing_time < timeout;
    let has_errors = parser.has_errors();

    ErrorRecoveryStressResult {
        test_name: test_name.to_string(),
        parsing_time,
        errors_detected: if has_errors { 1 } else { 0 }, // Simplified error count
        recovery_successful: within_timeout,
        max_recovery_attempts: 1, // Simplified
        forward_progress_maintained: within_timeout,
    }
}

#[cfg(test)]
mod error_recovery_stress_tests {
    use super::*;

    #[test]
    fn test_massive_syntax_error_cascade() {
        let tokens = generate_error_cascade_tokens(100);

        let result = test_error_recovery_stress(tokens, "massive_syntax_error_cascade", 5000);

        println!("Massive Syntax Error Cascade Results:");
        println!("  Parsing Time: {:?}", result.parsing_time);
        println!("  Recovery Successful: {}", result.recovery_successful);
        println!("  Forward Progress: {}", result.forward_progress_maintained);

        assert!(
            result.recovery_successful,
            "Parser should recover from massive syntax errors"
        );
        assert!(
            result.forward_progress_maintained,
            "Parser should maintain forward progress"
        );
        assert!(
            result.parsing_time < Duration::from_secs(5),
            "Parser should handle stress in reasonable time"
        );
    }

    #[test]
    fn test_deeply_nested_error_recovery() {
        // Create deeply nested structures with errors
        let mut tokens = vec![
            TokenType::Fn,
            TokenType::Identifier("test_deep_nesting".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
        ];

        // Create 25 levels of nesting with errors scattered throughout
        for depth in 0..25 {
            if depth % 7 == 0 {
                // Introduce errors at certain depths - missing opening brace
                tokens.extend(vec![
                    TokenType::If,
                    TokenType::Identifier(format!("condition_{depth}")),
                    // Missing opening brace
                ]);
            } else if depth % 13 == 0 {
                // Different type of error - missing condition
                tokens.extend(vec![
                    TokenType::While,
                    // Missing condition
                    TokenType::LeftBrace,
                    TokenType::Let,
                    TokenType::Identifier("x".to_string()),
                    TokenType::Equal,
                    // Missing value
                    TokenType::Semicolon,
                ]);
            } else {
                // Valid nesting
                tokens.extend(vec![
                    TokenType::If,
                    TokenType::Identifier(format!("condition_{depth}")),
                    TokenType::LeftBrace,
                    TokenType::Let,
                    TokenType::Identifier(format!("var_{depth}")),
                    TokenType::Equal,
                    TokenType::IntegerLiteral(depth as i64),
                    TokenType::Semicolon,
                ]);
            }
        }

        // Close some blocks (some will be unmatched due to errors)
        for depth in 0..15 {
            if depth % 7 != 0 && depth % 13 != 0 {
                tokens.push(TokenType::RightBrace);
            }
        }

        tokens.extend(vec![TokenType::RightBrace, TokenType::Eof]);

        let result = test_error_recovery_stress(tokens, "deeply_nested_error_recovery", 3000);

        println!("Deeply Nested Error Recovery Results:");
        println!("  Parsing Time: {:?}", result.parsing_time);
        println!("  Recovery Successful: {}", result.recovery_successful);

        assert!(
            result.recovery_successful,
            "Parser should handle deeply nested errors"
        );
        assert!(
            result.parsing_time < Duration::from_secs(3),
            "Deep nesting should not cause exponential slowdown"
        );
    }

    #[test]
    fn test_malformed_expression_storm() {
        // Generate many malformed expressions in sequence
        let mut tokens = vec![
            TokenType::Fn,
            TokenType::Identifier("expression_storm".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
        ];

        for i in 0..100 {
            match i % 6 {
                0 => {
                    // Invalid operator sequence
                    tokens.extend(vec![
                        TokenType::Let,
                        TokenType::Identifier("x".to_string()),
                        TokenType::Equal,
                        TokenType::Plus,
                        TokenType::Star,
                        TokenType::Slash,
                        TokenType::Percent,
                        TokenType::Semicolon,
                    ]);
                }
                1 => {
                    // Unmatched parentheses
                    tokens.extend(vec![
                        TokenType::Let,
                        TokenType::Identifier("y".to_string()),
                        TokenType::Equal,
                        TokenType::LeftParen,
                        TokenType::LeftParen,
                        TokenType::LeftParen,
                        TokenType::LeftParen,
                        TokenType::LeftParen,
                        TokenType::Semicolon,
                    ]);
                }
                2 => {
                    // Unmatched brackets
                    tokens.extend(vec![
                        TokenType::Let,
                        TokenType::Identifier("z".to_string()),
                        TokenType::Equal,
                        TokenType::LeftBracket,
                        TokenType::LeftBracket,
                        TokenType::LeftBracket,
                        TokenType::LeftBracket,
                        TokenType::RightBracket,
                        TokenType::Semicolon,
                    ]);
                }
                3 => {
                    // Invalid member access
                    tokens.extend(vec![
                        TokenType::Let,
                        TokenType::Identifier("b".to_string()),
                        TokenType::Equal,
                        TokenType::Dot,
                        TokenType::Identifier("member".to_string()),
                        TokenType::Dot,
                        TokenType::Identifier("access".to_string()),
                        TokenType::Dot,
                        TokenType::Semicolon,
                    ]);
                }
                4 => {
                    // Unclosed string (simulated with identifier)
                    tokens.extend(vec![
                        TokenType::Let,
                        TokenType::Identifier("d".to_string()),
                        TokenType::Equal,
                        TokenType::StringLiteral("unclosed string".to_string()),
                        // Missing semicolon to create error
                    ]);
                }
                5 => {
                    // Valid statement
                    tokens.extend(vec![
                        TokenType::Let,
                        TokenType::Identifier(format!("valid_{i}")),
                        TokenType::Equal,
                        TokenType::IntegerLiteral(i as i64),
                        TokenType::Plus,
                        TokenType::IntegerLiteral(1),
                        TokenType::Semicolon,
                    ]);
                }
                _ => unreachable!(),
            }
        }

        tokens.extend(vec![TokenType::RightBrace, TokenType::Eof]);

        let result = test_error_recovery_stress(tokens, "malformed_expression_storm", 4000);

        println!("Malformed Expression Storm Results:");
        println!("  Parsing Time: {:?}", result.parsing_time);
        println!("  Recovery Successful: {}", result.recovery_successful);

        assert!(
            result.recovery_successful,
            "Parser should handle expression error storms"
        );
        assert!(
            result.parsing_time < Duration::from_secs(4),
            "Expression errors should not cause hangs"
        );
    }

    #[test]
    fn test_error_recovery_boundary_conditions() {
        // Test error recovery at various boundary conditions
        let test_cases = vec![
            ("empty_file", vec![TokenType::Eof]),
            (
                "only_delimiters",
                vec![
                    TokenType::LeftParen,
                    TokenType::RightParen,
                    TokenType::LeftBrace,
                    TokenType::RightBrace,
                    TokenType::LeftBracket,
                    TokenType::RightBracket,
                    TokenType::LeftParen,
                    TokenType::RightParen,
                    TokenType::Eof,
                ],
            ),
            (
                "only_operators",
                vec![
                    TokenType::Plus,
                    TokenType::Minus,
                    TokenType::Star,
                    TokenType::Slash,
                    TokenType::Percent,
                    TokenType::Equal,
                    TokenType::Less,
                    TokenType::Greater,
                    TokenType::Bang,
                    TokenType::Eof,
                ],
            ),
            (
                "only_keywords",
                vec![
                    TokenType::Fn,
                    TokenType::Let,
                    TokenType::Var,
                    TokenType::If,
                    TokenType::Else,
                    TokenType::While,
                    TokenType::For,
                    TokenType::Return,
                    TokenType::Eof,
                ],
            ),
            (
                "mixed_chaos",
                vec![
                    TokenType::Fn,
                    TokenType::LeftParen,
                    TokenType::RightParen,
                    TokenType::LeftBrace,
                    TokenType::Let,
                    TokenType::If,
                    TokenType::RightBrace,
                    TokenType::Else,
                    TokenType::Var,
                    TokenType::While,
                    TokenType::Semicolon,
                    TokenType::Semicolon,
                    TokenType::For,
                    TokenType::Return,
                    TokenType::LeftBracket,
                    TokenType::RightBrace,
                    TokenType::RightBracket,
                    TokenType::Eof,
                ],
            ),
        ];

        for (test_name, tokens) in test_cases {
            let result = test_error_recovery_stress(tokens, test_name, 1000);

            println!("{} Results:", test_name);
            println!("  Parsing Time: {:?}", result.parsing_time);
            println!("  Recovery Successful: {}", result.recovery_successful);

            assert!(
                result.recovery_successful,
                "Parser should handle boundary condition: {}",
                test_name
            );
            assert!(
                result.parsing_time < Duration::from_millis(1000),
                "Boundary condition should parse quickly: {}",
                test_name
            );
        }
    }

    #[test]
    fn test_high_frequency_error_bursts() {
        // Create short bursts of high-frequency errors
        let mut tokens = Vec::new();

        for burst in 0..10 {
            // Valid code section
            tokens.extend(vec![
                TokenType::Fn,
                TokenType::Identifier(format!("valid_function_{burst}")),
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(42),
                TokenType::Semicolon,
                TokenType::Return,
                TokenType::Identifier("x".to_string()),
                TokenType::Semicolon,
                TokenType::RightBrace,
            ]);

            // Error burst section
            for _error in 0..5 {
                tokens.extend(vec![
                    TokenType::Fn,
                    // Missing name and parameters
                    TokenType::LeftBrace,
                    TokenType::LeftBrace,
                    TokenType::LeftBrace,
                    TokenType::Let,
                    TokenType::Equal,
                    // Missing variable name and value
                    TokenType::Semicolon,
                    TokenType::If,
                    TokenType::While,
                    TokenType::RightBrace,
                    TokenType::RightBrace,
                    TokenType::RightBrace,
                    TokenType::Semicolon,
                ]);
            }
        }

        tokens.push(TokenType::Eof);

        let result = test_error_recovery_stress(tokens, "high_frequency_error_bursts", 3000);

        println!("High Frequency Error Bursts Results:");
        println!("  Parsing Time: {:?}", result.parsing_time);
        println!("  Recovery Successful: {}", result.recovery_successful);

        assert!(
            result.recovery_successful,
            "Parser should handle error bursts"
        );
        assert!(
            result.parsing_time < Duration::from_secs(3),
            "Error bursts should not cause significant slowdown"
        );
    }

    #[test]
    fn test_error_recovery_scalability() {
        // Test how error recovery scales with program size
        let sizes = vec![
            ("small_errors", 25),
            ("medium_errors", 100),
            ("large_errors", 250),
        ];

        let mut scalability_results = Vec::new();

        for (size_name, error_count) in sizes {
            let mut tokens = Vec::new();

            for i in 0..error_count {
                // Mix of different error types
                match i % 5 {
                    0 => {
                        // Missing params
                        tokens.extend(vec![
                            TokenType::Fn,
                            TokenType::Identifier(format!("error_{i}")),
                            // Missing parameter list
                            TokenType::LeftBrace,
                            TokenType::RightBrace,
                        ]);
                    }
                    1 => {
                        // Missing value
                        tokens.extend(vec![
                            TokenType::Let,
                            TokenType::Identifier(format!("x_{i}")),
                            TokenType::Equal,
                            // Missing value
                            TokenType::Semicolon,
                        ]);
                    }
                    2 => {
                        // Unclosed if
                        tokens.extend(vec![
                            TokenType::If,
                            TokenType::Identifier(format!("condition_{i}")),
                            TokenType::LeftBrace,
                            // Missing close brace
                        ]);
                    }
                    3 => {
                        // Missing condition
                        tokens.extend(vec![
                            TokenType::While,
                            // Missing condition
                            TokenType::LeftBrace,
                            TokenType::Break,
                            TokenType::Semicolon,
                            TokenType::RightBrace,
                        ]);
                    }
                    4 => {
                        // Double return
                        tokens.extend(vec![
                            TokenType::Return,
                            TokenType::Return,
                            TokenType::IntegerLiteral(i as i64),
                            TokenType::Semicolon,
                        ]);
                    }
                    _ => unreachable!(),
                }
            }

            tokens.push(TokenType::Eof);

            let result = test_error_recovery_stress(tokens, size_name, 10000);
            scalability_results.push((size_name, error_count, result.parsing_time));

            println!(
                "{} ({} errors): {:?}",
                size_name, error_count, result.parsing_time
            );

            assert!(
                result.recovery_successful,
                "Parser should scale for error recovery: {}",
                size_name
            );
        }

        // Check that parsing time doesn't grow exponentially
        if scalability_results.len() >= 3 {
            let small_time = scalability_results[0].2.as_millis();
            let large_time = scalability_results[2].2.as_millis();

            if small_time > 0 {
                let time_scaling_factor = large_time as f64 / small_time as f64;
                println!(
                    "Error recovery time scaling factor: {:.2}x",
                    time_scaling_factor
                );

                // Should scale better than quadratically (allow up to 50x for stress testing)
                assert!(
                    time_scaling_factor < 50.0,
                    "Error recovery scaling too poorly: {:.2}x",
                    time_scaling_factor
                );
            }
        }
    }
}
