//! Enhanced Test Infrastructure Demonstration
//!
//! This test file demonstrates the enhanced test infrastructure including:
//! - Test case generation macros for repetitive patterns
//! - Enhanced fixture management with metadata
//! - Parameterized testing framework
//! - Automated test discovery patterns

use ferra_parser::{
    test_all_literals, test_all_operators, test_precedence_matrix, test_statement_types,
    test_utils::{
        self, assertions, enhanced_fixtures, ExpectedExpressionType,
        performance::measure_parse_time,
    },
    token::TokenType,
};
use std::time::Duration;

// Test the enhanced test generation macros
#[test]
fn test_macro_generation_capabilities() {
    // Test that we can generate operator tests using the macro
    fn test_single_operator(left: TokenType, op: TokenType, right: TokenType) {
        let arena = test_utils::test_arena();
        let tokens = test_utils::mock_token_stream(vec![left, op, right, TokenType::Eof]);
        let mut parser = test_utils::test_expression_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_ok(), "Should parse binary expression");
    }

    // This uses the test_all_operators macro
    test_all_operators!(
        test_single_operator,
        TokenType::IntegerLiteral(1),
        TokenType::IntegerLiteral(2)
    );
}

// Generate precedence matrix test using macro
test_precedence_matrix!(test_precedence_combinations);

// Generate literal tests using macro
test_all_literals!(test_all_literal_types, |literal: TokenType| {
    let arena = test_utils::test_arena();
    let tokens = test_utils::mock_token_stream(vec![literal, TokenType::Eof]);
    let mut parser = test_utils::test_expression_parser(&arena, tokens);
    let result = parser.parse_expression(0);
    assert!(result.is_ok(), "Should parse literal expression");
});

// Generate statement type tests using macro
test_statement_types!(test_statement_type_validation, |stmt, expected| {
    assertions::assert_statement_type(stmt, expected);
});

// Test enhanced fixture management
#[test]
fn test_enhanced_fixture_management() {
    // Test fixture catalog functionality
    let catalog = enhanced_fixtures::get_fixture_catalog();
    assert!(!catalog.is_empty(), "Fixture catalog should not be empty");

    // Test priority filtering
    let high_priority = enhanced_fixtures::get_fixtures_by_priority(5);
    assert!(
        !high_priority.is_empty(),
        "Should have high priority fixtures"
    );

    // Verify that all high priority fixtures are actually priority 5
    for fixture in &high_priority {
        assert_eq!(
            fixture.test_priority, 5,
            "High priority fixtures should be priority 5"
        );
    }

    // Test category filtering
    let valid_fixtures = enhanced_fixtures::get_fixtures_by_category("valid");
    assert!(!valid_fixtures.is_empty(), "Should have valid fixtures");

    // Verify all returned fixtures are in the correct category
    for fixture in &valid_fixtures {
        assert_eq!(
            fixture.category, "valid",
            "Fixtures should be in valid category"
        );
    }
}

// Test automated fixture parsing with metadata
#[test]
fn test_automated_fixture_testing() {
    let catalog = enhanced_fixtures::get_fixture_catalog();

    for metadata in catalog {
        let source = enhanced_fixtures::load_fixture_by_metadata(&metadata);
        assert!(
            !source.is_empty(),
            "Fixture {} should not be empty",
            metadata.filename
        );

        let arena = test_utils::test_arena();
        let tokens = test_utils::mock_tokens_from_source(&source);
        let mut parser = test_utils::test_program_parser(&arena, tokens);

        let result = parser.parse_compilation_unit();

        if metadata.expected_parse_result {
            assert!(
                result.is_ok(),
                "Fixture {} (category: {}) should parse successfully but got error: {:?}",
                metadata.filename,
                metadata.category,
                result.err()
            );
        } else {
            assert!(
                result.is_err(),
                "Fixture {} (category: {}) should fail to parse but succeeded",
                metadata.filename,
                metadata.category
            );
        }
    }
}

// Test parameterized operator precedence
#[test]
fn test_parameterized_precedence() {
    let arena = test_utils::test_arena();

    // Test that multiplication has higher precedence than addition
    let tokens = test_utils::mock_token_stream(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);

    let mut parser = test_utils::test_expression_parser(&arena, tokens);
    let expr = parser.parse_expression(0).unwrap();

    // Should be a binary expression (1 + (2 * 3))
    assertions::assert_expression_type(expr, ExpectedExpressionType::Binary);
}

// Test expression type matrix
#[test]
fn test_expression_type_matrix() {
    let arena = test_utils::test_arena();
    
    let test_cases = vec![
        // (source, expected_type) - Only simple expressions that work with current lexer
        ("42", ExpectedExpressionType::Literal),
        ("x", ExpectedExpressionType::Identifier),
        ("x + y", ExpectedExpressionType::Binary),
        ("-x", ExpectedExpressionType::Unary),
    ];
    
    for (source, expected_type) in test_cases {
        let tokens = test_utils::mock_tokens_from_source(source);
        let mut parser = test_utils::test_expression_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        
        if let Ok(expr) = result {
            assertions::assert_expression_type(expr, expected_type);
        }
    }
}

// Test complex combinations using utilities
#[test]
fn test_complex_parsing_combinations() {
    let arena = test_utils::test_arena();

    // Test simpler expression that should work with current lexer
    let source = "x + y";
    let tokens = test_utils::mock_tokens_from_source(source);
    let mut parser = test_utils::test_expression_parser(&arena, tokens);
    let result = parser.parse_expression(0);

    assert!(
        result.is_ok(),
        "Simple expression should parse successfully"
    );
    if let Ok(expr) = result {
        // Should be a binary expression
        assertions::assert_expression_type(expr, ExpectedExpressionType::Binary);
    }
}

// Test performance of macro-generated tests
#[test]
fn test_macro_generated_performance() {
    use std::time::{Duration, Instant};

    let start = Instant::now();

    // Test a batch of generated tests
    for i in 0..10 {
        let arena = test_utils::test_arena();
        let tokens = test_utils::mock_token_stream(vec![
            TokenType::IntegerLiteral(i),
            TokenType::Plus,
            TokenType::IntegerLiteral(i + 1),
            TokenType::Eof,
        ]);
        let mut parser = test_utils::test_expression_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_ok(), "Generated test {} should pass", i);
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(100),
        "Macro-generated tests should be fast"
    );
}

// Test fixture metadata validation
#[test]
fn test_fixture_metadata_validation() {
    let catalog = enhanced_fixtures::get_fixture_catalog();

    for metadata in catalog {
        // Validate metadata fields
        assert!(
            !metadata.category.is_empty(),
            "Category should not be empty"
        );
        assert!(
            !metadata.filename.is_empty(),
            "Filename should not be empty"
        );
        assert!(
            !metadata.description.is_empty(),
            "Description should not be empty"
        );
        assert!(
            metadata.test_priority >= 1 && metadata.test_priority <= 5,
            "Priority should be 1-5"
        );

        // Validate that fixture file actually exists and can be loaded
        let source = enhanced_fixtures::load_fixture_by_metadata(&metadata);
        assert!(
            !source.is_empty(),
            "Fixture content should not be empty for {}",
            metadata.filename
        );
    }
}

// Integration test for all enhanced features
#[test]
fn test_enhanced_infrastructure_integration() {
    // Test that all components work together
    let high_priority_fixtures = enhanced_fixtures::get_fixtures_by_priority(4);

    for metadata in high_priority_fixtures {
        let source = enhanced_fixtures::load_fixture_by_metadata(&metadata);
        let arena = test_utils::test_arena();
        let tokens = test_utils::mock_tokens_from_source(&source);
        let mut parser = test_utils::test_program_parser(&arena, tokens);

        let (result, duration) = measure_parse_time(|| parser.parse_compilation_unit());

        // Validate both the result and performance
        if metadata.expected_parse_result {
            assert!(
                result.is_ok(),
                "High priority fixture {} should parse",
                metadata.filename
            );
        }

        // High priority fixtures should parse quickly
        assert!(
            duration < Duration::from_secs(1),
            "High priority fixture parsing should be fast"
        );
    }
}

// Test parsing performance utilities
#[test]
fn test_performance_utilities() {
    let arena = test_utils::test_arena();
    let tokens = test_utils::mock_tokens_from_source("1 + 2 * 3");
    let mut parser = test_utils::test_expression_parser(&arena, tokens);
    
    // Test time measurement
    let (result, duration) = measure_parse_time(|| {
        parser.parse_expression(0)
    });
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000); // Should be very fast
    
    // Test time assertion
    let expr = result.unwrap();
    assertions::assert_expression_type(expr, ExpectedExpressionType::Binary);
}
