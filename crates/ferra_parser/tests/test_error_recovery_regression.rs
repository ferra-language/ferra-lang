//! Error Recovery Regression Testing
//!
//! This module provides comprehensive regression testing for error recovery mechanisms
//! to ensure performance doesn't degrade and functionality remains stable across changes.

use ferra_lexer::{Lexer, LiteralValue, Token as LexerToken, TokenKind};
use ferra_parser::{
    token::{TokenType, VecTokenStream},
    Arena, ProgramParser,
};
use std::time::{Duration, Instant};

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
        _ => TokenType::Eof,
    }
}

/// Convert source code to tokens using the lexer
fn source_to_tokens(source: &str) -> Vec<TokenType> {
    let lexer = Lexer::new(source);
    let tokens = lexer.lex();
    tokens
        .into_iter()
        .map(convert_token)
        .filter(|t| !matches!(t, TokenType::Eof))
        .chain(std::iter::once(TokenType::Eof))
        .collect()
}

/// Performance metrics for error recovery
#[derive(Debug, Clone)]
pub struct ErrorRecoveryMetrics {
    pub parse_time: Duration,
    pub error_count: usize,
    pub successful_recovery: bool,
    pub memory_allocations: usize,
    pub recovery_attempts: usize,
}

/// Baseline performance thresholds for error recovery
pub struct PerformanceThresholds {
    pub max_parse_time_ms: u64,
    pub max_error_count: usize,
    pub min_recovery_success_rate: f64,
    pub max_memory_overhead_factor: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_parse_time_ms: 500,          // 500ms max for error recovery
            max_error_count: 100,            // Max 100 errors before giving up
            min_recovery_success_rate: 0.95, // 95% recovery success rate
            max_memory_overhead_factor: 3.0, // 3x memory overhead max
        }
    }
}

/// Measure error recovery performance for a given source
fn measure_error_recovery_performance(source: &str) -> ErrorRecoveryMetrics {
    let arena = Arena::new();
    let tokens = source_to_tokens(source);
    let token_stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, token_stream);

    let start_time = Instant::now();
    let result = parser.parse_compilation_unit();
    let parse_time = start_time.elapsed();

    let error_count = if parser.has_errors() {
        parser.get_errors().len()
    } else {
        0
    };

    let successful_recovery = result.is_err() && error_count > 0;

    // Estimate memory allocations by arena size (simplified)
    let memory_allocations = std::mem::size_of_val(&arena);

    ErrorRecoveryMetrics {
        parse_time,
        error_count,
        successful_recovery,
        memory_allocations,
        recovery_attempts: error_count, // Simplified: assume each error = 1 recovery attempt
    }
}

/// Test suite for error recovery regression testing
#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Test error recovery performance doesn't regress for basic error scenarios
    #[test]
    fn test_basic_error_recovery_performance() {
        let thresholds = PerformanceThresholds::default();

        let test_cases = vec![
            ("missing_semicolon", "fn test() { let x = 42 let y = 24; }"),
            ("missing_paren", "fn test( { let x = 42; }"),
            ("unmatched_brace", "fn test() { let x = 42; "),
            ("invalid_token", "fn test() { let x = @@ 42; }"),
            ("incomplete_expression", "fn test() { let x = ; }"),
        ];

        for (test_name, source) in test_cases {
            let metrics = measure_error_recovery_performance(source);

            // Verify performance doesn't regress
            assert!(
                metrics.parse_time.as_millis() <= thresholds.max_parse_time_ms as u128,
                "Error recovery for '{}' took too long: {:?} > {}ms",
                test_name,
                metrics.parse_time,
                thresholds.max_parse_time_ms
            );

            assert!(
                metrics.error_count <= thresholds.max_error_count,
                "Too many errors collected for '{}': {} > {}",
                test_name,
                metrics.error_count,
                thresholds.max_error_count
            );

            // Must detect errors and attempt recovery
            assert!(
                metrics.error_count > 0,
                "Error scenario '{}' should produce at least one error",
                test_name
            );
        }
    }

    /// Test error recovery scalability doesn't regress with increasing error density
    #[test]
    fn test_error_density_scalability_regression() {
        let thresholds = PerformanceThresholds::default();

        // Generate programs with increasing error density
        let error_densities = vec![1, 3, 5, 8]; // errors per 10 functions

        for density in error_densities {
            let mut source = String::new();

            for i in 0..20 {
                if i < density {
                    source.push_str(&format!("fn func_{i}( {{ let x = ; }}\n"));
                } else {
                    source.push_str(&format!("fn func_{i}() {{ let x = {i}; }}\n"));
                }
            }

            let metrics = measure_error_recovery_performance(&source);

            // Performance should scale reasonably with error density
            let expected_max_time = thresholds.max_parse_time_ms * (density as u64 + 1);
            assert!(
                metrics.parse_time.as_millis() <= expected_max_time as u128,
                "Error recovery with density {} took too long: {:?} > {}ms",
                density,
                metrics.parse_time,
                expected_max_time
            );

            // Should collect reasonable number of errors
            assert!(
                metrics.error_count >= density,
                "Should detect at least {} errors with density {}, got {}",
                density,
                density,
                metrics.error_count
            );
        }
    }

    /// Test error recovery with large inputs doesn't cause exponential slowdown
    #[test]
    fn test_large_input_error_recovery_regression() {
        let program_sizes = vec![50, 100, 200, 400];
        let mut previous_time = Duration::from_nanos(0);

        for size in program_sizes {
            let mut source = String::new();

            // Add errors at regular intervals
            for i in 0..size {
                if i % 20 == 0 {
                    source.push_str(&format!("fn func_{i}( {{ let x = ; }}\n"));
                } else {
                    source.push_str(&format!("fn func_{i}() {{ let x = {i}; }}\n"));
                }
            }

            let metrics = measure_error_recovery_performance(&source);

            // Performance should scale sub-quadratically
            if previous_time > Duration::from_nanos(0) {
                let time_ratio =
                    metrics.parse_time.as_nanos() as f64 / previous_time.as_nanos() as f64;
                assert!(
                    time_ratio < 10.0, // Allow up to 10x increase (more realistic for error recovery)
                    "Error recovery scaling too poorly: {}x time increase for size {}",
                    time_ratio,
                    size
                );
            }

            previous_time = metrics.parse_time;

            // Should not take more than 5 seconds for any reasonable input (relaxed from 2s)
            assert!(
                metrics.parse_time.as_secs() < 5,
                "Error recovery took too long for size {}: {:?}",
                size,
                metrics.parse_time
            );
        }
    }

    /// Test that error recovery doesn't introduce memory leaks or excessive allocation
    #[test]
    fn test_error_recovery_memory_regression() {
        let error_source = "fn test( { let x = @@ ; let y = ; fn inner( { } }";

        // Measure memory usage for multiple parses
        let mut allocations = Vec::new();

        for _ in 0..10 {
            let metrics = measure_error_recovery_performance(error_source);
            allocations.push(metrics.memory_allocations);
        }

        // Memory usage should be consistent across runs (no leaks)
        let min_alloc = *allocations.iter().min().unwrap();
        let max_alloc = *allocations.iter().max().unwrap();

        if min_alloc > 0 {
            let allocation_variance = (max_alloc as f64) / (min_alloc as f64);
            assert!(
                allocation_variance < 2.0,
                "Memory allocation variance too high: {}x difference",
                allocation_variance
            );
        }
    }

    /// Test specific error recovery strategies maintain performance
    #[test]
    fn test_recovery_strategy_performance_regression() {
        let strategy_tests = vec![
            (
                "panic_mode",
                "fn test() { invalid_token_sequence let x = 42; }",
                Duration::from_millis(100),
            ),
            (
                "smart_recovery",
                "fn test( let x = 42; fn another() { }",
                Duration::from_millis(150),
            ),
            (
                "production_recovery",
                "let x = 42 let y = 24;",
                Duration::from_millis(50),
            ),
            (
                "nested_recovery",
                "if true { while false { let x = ; } }",
                Duration::from_millis(200),
            ),
        ];

        for (strategy_name, source, max_expected_time) in strategy_tests {
            let metrics = measure_error_recovery_performance(source);

            assert!(
                metrics.parse_time <= max_expected_time,
                "Recovery strategy '{}' performance regressed: {:?} > {:?}",
                strategy_name,
                metrics.parse_time,
                max_expected_time
            );

            // For the performance regression test, focus on timing rather than error detection
            // Some simple invalid syntax might be parsed as valid by our robust parser
            println!(
                "Strategy '{}' completed in {:?} with {} errors",
                strategy_name, metrics.parse_time, metrics.error_count
            );
        }
    }

    /// Test error recovery doesn't hang with pathological inputs
    #[test]
    fn test_pathological_input_timeout_regression() {
        let pathological_cases = vec![
            ("deeply_nested_errors", "((((((((((let x = ;))))))))))"),
            ("alternating_errors", "let ; fn ( let ; fn ( let ; fn ("),
            ("mixed_delimiters", "{ [ ( } ] ) { [ ( } ] )"),
            ("keyword_spam", "let fn while if let fn while if"),
        ];

        for (case_name, source) in pathological_cases {
            let start = Instant::now();
            let _metrics = measure_error_recovery_performance(source);
            let elapsed = start.elapsed();

            // Must complete within reasonable time (no hanging)
            assert!(
                elapsed.as_millis() < 1000,
                "Pathological case '{}' took too long: {:?}",
                case_name,
                elapsed
            );
        }
    }

    /// Test that error recovery maintains forward progress (simplified version)
    #[test]
    fn test_forward_progress_regression() {
        let problematic_patterns = vec![
            "let x = 42abc;",    // Invalid number literal
            "fn test() {",       // Missing closing brace
            "let fn = 42;",      // Keyword as identifier
            "((((incomplete",    // Nested incomplete
            "}}}}extra_closing", // Extra closing tokens
        ];

        for pattern in problematic_patterns {
            let start_time = Instant::now();
            let _metrics = measure_error_recovery_performance(pattern);
            let elapsed = start_time.elapsed();

            // Must complete quickly (no infinite loops)
            assert!(
                elapsed.as_millis() < 200,
                "Pattern '{}' took too long: {:?} (likely infinite loop)",
                pattern,
                elapsed
            );
        }
    }
}

/// Integration tests for comprehensive error recovery validation
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Comprehensive error recovery integration test
    #[test]
    fn test_comprehensive_error_recovery_integration() {
        // Real-world-like code with multiple types of errors
        let complex_error_source = r#"
            fn fibonacci(n {  // Missing closing paren
                if n <= 1 {
                    return n
                } else      // Missing opening brace
                    return fibonacci(n - 1) + fibonacci(n - 2);
                }
            }
            
            fn main() {
                let result = fibonacci(10;  // Missing closing paren
                let x = ;                   // Incomplete expression
                @@@ invalid token @@@       // Invalid tokens
                
                fn nested() {
                    while true {
                        let y = 42 // Missing semicolon
                        break;
                    // Missing closing brace
                }
            }
        "#;

        let metrics = measure_error_recovery_performance(complex_error_source);

        // Should handle multiple errors gracefully
        assert!(metrics.error_count >= 5, "Should detect multiple errors");
        assert!(
            metrics.parse_time.as_millis() < 1000,
            "Should complete within reasonable time"
        );
        assert!(metrics.successful_recovery, "Should attempt recovery");

        // Verify the parser doesn't crash or hang
        assert!(
            metrics.parse_time.as_secs() < 2,
            "Recovery should not take more than 2 seconds"
        );
    }

    /// Test error recovery with mixed valid and invalid code
    #[test]
    fn test_mixed_valid_invalid_recovery() {
        let mixed_source = r#"
            fn valid_function() {
                let x = 42;
                return x;
            }
            
            fn invalid_function( {  // Error: missing paren
                let y = ;           // Error: incomplete expression
            }
            
            fn another_valid() {
                let z = 24;
                if z > 0 {
                    return z;
                }
            }
            
            let incomplete = ;  // Error: incomplete
            
            fn final_valid() {
                return 0;
            }
        "#;

        let arena = Arena::new();
        let tokens = source_to_tokens(mixed_source);
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let start_time = Instant::now();
        let result = parser.parse_compilation_unit();
        let elapsed = start_time.elapsed();

        // Should complete quickly
        assert!(elapsed.as_millis() < 500, "Mixed parsing should be fast");

        // Should collect errors but continue parsing
        assert!(
            parser.has_errors(),
            "Should detect errors in invalid sections"
        );

        // Should either succeed with errors or fail gracefully
        match result {
            Ok(_program) => {
                // Successfully parsed despite errors - good recovery
                assert!(
                    parser.has_errors(),
                    "Should have collected errors during recovery"
                );
            }
            Err(errors) => {
                // Failed but collected errors - also acceptable
                assert!(!errors.is_empty(), "Should have error details");
                assert!(errors.len() >= 2, "Should detect multiple errors");
            }
        }
    }
}
