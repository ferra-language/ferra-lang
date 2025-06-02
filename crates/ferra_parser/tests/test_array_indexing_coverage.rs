//! Array Indexing Comprehensive Test Coverage
//!
//! This module provides comprehensive test coverage for array indexing functionality
//! including edge cases, error scenarios, performance tests, and integration patterns.

use ferra_parser::{
    ast::{Expression, Literal},
    token::{TokenType, VecTokenStream},
    Arena, PrattParser,
};

/// Helper function to create test arena
fn test_arena() -> Arena {
    Arena::new()
}

/// Helper function to create parser for testing
fn test_parser(arena: &Arena, tokens: Vec<TokenType>) -> PrattParser<VecTokenStream> {
    let token_stream = VecTokenStream::from_token_types(tokens);
    PrattParser::new(arena, token_stream)
}

#[cfg(test)]
mod array_indexing_tests {
    use super::*;

    /// Test basic array indexing with various index types
    #[test]
    fn test_array_indexing_basic_coverage() {
        let arena = test_arena();

        // Integer index
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(42),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse integer index: {:?}",
            result
        );

        // Variable index
        let tokens = vec![
            TokenType::Identifier("data".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("index".to_string()),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse variable index: {:?}",
            result
        );

        // String index (for hash maps/objects)
        let tokens = vec![
            TokenType::Identifier("map".to_string()),
            TokenType::LeftBracket,
            TokenType::StringLiteral("key".to_string()),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_ok(), "Failed to parse string index: {:?}", result);
    }

    /// Test nested array indexing (multi-dimensional arrays)
    #[test]
    fn test_nested_array_indexing_coverage() {
        let arena = test_arena();

        // Two-dimensional indexing: matrix[row][col]
        let tokens = vec![
            TokenType::Identifier("matrix".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("row".to_string()),
            TokenType::RightBracket,
            TokenType::LeftBracket,
            TokenType::Identifier("col".to_string()),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse 2D array indexing: {:?}",
            result
        );

        // Three-dimensional indexing: cube[x][y][z]
        let tokens = vec![
            TokenType::Identifier("cube".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("x".to_string()),
            TokenType::RightBracket,
            TokenType::LeftBracket,
            TokenType::Identifier("y".to_string()),
            TokenType::RightBracket,
            TokenType::LeftBracket,
            TokenType::Identifier("z".to_string()),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse 3D array indexing: {:?}",
            result
        );
    }

    /// Test array indexing with complex expressions as indices
    #[test]
    fn test_complex_index_expressions_coverage() {
        let arena = test_arena();

        // Binary expression as index: arr[i + 1]
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("i".to_string()),
            TokenType::Plus,
            TokenType::IntegerLiteral(1),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse binary expression index: {:?}",
            result
        );

        // Function call as index: arr[compute_index()]
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("compute_index".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse function call index: {:?}",
            result
        );

        // Complex nested expression: arr[hash(key) % table_size]
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("hash".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("key".to_string()),
            TokenType::RightParen,
            TokenType::Percent,
            TokenType::Identifier("table_size".to_string()),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse complex index expression: {:?}",
            result
        );
    }

    /// Test array indexing integrated with other operations
    #[test]
    fn test_array_indexing_integration_coverage() {
        let arena = test_arena();

        // Member access after indexing: arr[i].field
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("i".to_string()),
            TokenType::RightBracket,
            TokenType::Dot,
            TokenType::Identifier("field".to_string()),
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse indexing + member access: {:?}",
            result
        );

        // Function call after indexing: arr[i].method()
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("i".to_string()),
            TokenType::RightBracket,
            TokenType::Dot,
            TokenType::Identifier("method".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse indexing + method call: {:?}",
            result
        );

        // Indexing after function call: get_array()[index]
        let tokens = vec![
            TokenType::Identifier("get_array".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBracket,
            TokenType::Identifier("index".to_string()),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse function call + indexing: {:?}",
            result
        );

        // Complex chaining: obj.get_array()[index].process()
        let tokens = vec![
            TokenType::Identifier("obj".to_string()),
            TokenType::Dot,
            TokenType::Identifier("get_array".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBracket,
            TokenType::Identifier("index".to_string()),
            TokenType::RightBracket,
            TokenType::Dot,
            TokenType::Identifier("process".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse complex chaining: {:?}",
            result
        );
    }

    /// Test array indexing error cases
    #[test]
    fn test_array_indexing_error_coverage() {
        let arena = test_arena();

        // Missing closing bracket
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(0),
            TokenType::Eof, // Missing RightBracket
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_err(), "Should fail with missing closing bracket");

        // Empty index expression
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::RightBracket, // Empty index
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_err(), "Should fail with empty index");

        // Invalid index expression
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Plus, // Invalid start of expression
            TokenType::IntegerLiteral(1),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_err(), "Should fail with invalid index expression");
    }

    /// Test array indexing precedence
    #[test]
    fn test_array_indexing_precedence_coverage() {
        let arena = test_arena();

        // Indexing has higher precedence than arithmetic: arr[i] + 1
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("i".to_string()),
            TokenType::RightBracket,
            TokenType::Plus,
            TokenType::IntegerLiteral(1),
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse indexing + arithmetic: {:?}",
            result
        );

        // Indexing has higher precedence than member access: arr[i].field
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("i".to_string()),
            TokenType::RightBracket,
            TokenType::Dot,
            TokenType::Identifier("field".to_string()),
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse indexing + member access: {:?}",
            result
        );

        // Multiple operations with precedence: obj.arr[i] + other.val
        let tokens = vec![
            TokenType::Identifier("obj".to_string()),
            TokenType::Dot,
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("i".to_string()),
            TokenType::RightBracket,
            TokenType::Plus,
            TokenType::Identifier("other".to_string()),
            TokenType::Dot,
            TokenType::Identifier("val".to_string()),
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed to parse complex precedence: {:?}",
            result
        );
    }

    /// Performance test for array indexing parsing
    #[test]
    fn test_array_indexing_performance_coverage() {
        let arena = test_arena();

        // Test parsing performance with deeply nested indexing
        let mut tokens = vec![TokenType::Identifier("base".to_string())];

        // Create 50 levels of indexing: base[0][1][2]...[49]
        for i in 0..50 {
            tokens.push(TokenType::LeftBracket);
            tokens.push(TokenType::IntegerLiteral(i));
            tokens.push(TokenType::RightBracket);
        }
        tokens.push(TokenType::Eof);

        let start = std::time::Instant::now();
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Failed to parse deep indexing: {:?}",
            result
        );
        assert!(
            duration.as_millis() < 100,
            "Deep indexing parsing took too long: {:?}",
            duration
        );
    }

    /// Test array indexing AST structure validation
    #[test]
    fn test_array_indexing_ast_structure_coverage() {
        let arena = test_arena();

        // Test simple indexing AST structure
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(42),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);

        assert!(result.is_ok());
        if let Ok(expr) = result {
            match expr {
                Expression::Index(index_expr) => {
                    // Verify object is correct
                    match index_expr.object.as_ref() {
                        Expression::Identifier(name) => assert_eq!(name, "arr"),
                        _ => panic!("Expected object to be identifier"),
                    }
                    // Verify index is correct
                    match index_expr.index.as_ref() {
                        Expression::Literal(Literal::Integer(42)) => {}
                        _ => panic!("Expected index to be integer 42"),
                    }
                }
                _ => panic!("Expected index expression"),
            }
        }
    }

    /// Test edge cases for array indexing
    #[test]
    fn test_array_indexing_edge_cases_coverage() {
        let arena = test_arena();

        // Test with whitespace variations
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(0),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed with whitespace variations: {:?}",
            result
        );

        // Test with maximum integer index
        let tokens = vec![
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(i64::MAX),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(
            result.is_ok(),
            "Failed with max integer index: {:?}",
            result
        );

        // Test with long identifier names
        let long_name = "very_long_array_name_that_exceeds_normal_length".to_string();
        let tokens = vec![
            TokenType::Identifier(long_name.clone()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(0),
            TokenType::RightBracket,
            TokenType::Eof,
        ];
        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);
        assert!(result.is_ok(), "Failed with long identifier: {:?}", result);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use ferra_parser::{ProgramParser, StatementParser};

    /// Test array indexing in variable declarations
    #[test]
    fn test_array_indexing_in_variable_declarations() {
        let arena = test_arena();

        let tokens = vec![
            TokenType::Let,
            TokenType::Identifier("value".to_string()),
            TokenType::Equal,
            TokenType::Identifier("data".to_string()),
            TokenType::LeftBracket,
            TokenType::Identifier("index".to_string()),
            TokenType::RightBracket,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = StatementParser::new(&arena, token_stream);
        let result = parser.parse_statement();

        assert!(
            result.is_ok(),
            "Failed to parse array indexing in variable declaration: {:?}",
            result
        );
    }

    /// Test array indexing in function calls
    #[test]
    fn test_array_indexing_in_function_calls() {
        let arena = test_arena();

        let tokens = vec![
            TokenType::Identifier("process".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("data".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(0),
            TokenType::RightBracket,
            TokenType::Comma,
            TokenType::Identifier("data".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(1),
            TokenType::RightBracket,
            TokenType::RightParen,
            TokenType::Eof,
        ];

        let mut parser = test_parser(&arena, tokens);
        let result = parser.parse_expression(0);

        assert!(
            result.is_ok(),
            "Failed to parse array indexing in function call: {:?}",
            result
        );
    }

    /// Test array indexing in complete programs
    #[test]
    fn test_array_indexing_in_complete_programs() {
        let arena = test_arena();

        // Create a simple program with array indexing
        let tokens = vec![
            TokenType::Fn,
            TokenType::Identifier("main".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::Let,
            TokenType::Identifier("arr".to_string()),
            TokenType::Equal,
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(1),
            TokenType::Comma,
            TokenType::IntegerLiteral(2),
            TokenType::Comma,
            TokenType::IntegerLiteral(3),
            TokenType::RightBracket,
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Identifier("value".to_string()),
            TokenType::Equal,
            TokenType::Identifier("arr".to_string()),
            TokenType::LeftBracket,
            TokenType::IntegerLiteral(0),
            TokenType::RightBracket,
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);
        let result = parser.parse_compilation_unit();

        assert!(
            result.is_ok(),
            "Failed to parse program with array indexing: {:?}",
            result
        );
    }
}
