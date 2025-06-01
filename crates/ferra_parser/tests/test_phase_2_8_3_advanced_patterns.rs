//! Tests for Phase 2.8.3: Advanced Pattern Matching
//!
//! This module tests advanced pattern matching features including:
//! - Range patterns (1..=10)
//! - Slice patterns ([head, tail @ ..])  
//! - Or patterns (Some(x) | None)
//! - Guard patterns (x if x > 0)
//! - Binding patterns (name @ pattern)

use ferra_parser::{
    ast::{Arena, Pattern},
    pratt::parser::PrattParser,
    token::{stream::VecTokenStream, TokenType},
};

fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
    VecTokenStream::from_token_types(token_types)
}

#[test]
fn test_range_pattern_inclusive() {
    let arena = Arena::new();

    // Test 1..=10
    let tokens = create_token_stream(vec![
        TokenType::IntegerLiteral(1),
        TokenType::DotDotEqual,
        TokenType::IntegerLiteral(10),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Range(range) => {
                assert!(range.inclusive);
                assert!(range.start.is_some());
                assert!(range.end.is_some());
            }
            _ => panic!("Expected range pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_range_pattern_exclusive() {
    let arena = Arena::new();

    // Test 1..10
    let tokens = create_token_stream(vec![
        TokenType::IntegerLiteral(1),
        TokenType::DotDot,
        TokenType::IntegerLiteral(10),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Range(range) => {
                assert!(!range.inclusive);
                assert!(range.start.is_some());
                assert!(range.end.is_some());
            }
            _ => panic!("Expected range pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_slice_pattern_empty() {
    let arena = Arena::new();

    // Test []
    let tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Slice(slice) => {
                assert_eq!(slice.prefix.len(), 0);
                assert!(slice.rest.is_none());
                assert_eq!(slice.suffix.len(), 0);
            }
            _ => panic!("Expected slice pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_or_pattern_simple() {
    let arena = Arena::new();

    // Test Some | None
    let tokens = create_token_stream(vec![
        TokenType::Identifier("Some".to_string()),
        TokenType::Pipe,
        TokenType::Identifier("None".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Or(or_pattern) => {
                assert_eq!(or_pattern.patterns.len(), 2);
            }
            _ => panic!("Expected or pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_guard_pattern_simple() {
    let arena = Arena::new();

    // Test x if x > 0 (simplified)
    let tokens = create_token_stream(vec![
        TokenType::Identifier("x".to_string()),
        TokenType::If,
        TokenType::Identifier("x".to_string()),
        TokenType::Greater,
        TokenType::IntegerLiteral(0),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Guard(guard) => match guard.pattern.as_ref() {
                Pattern::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier pattern for guard base"),
            },
            _ => panic!("Expected guard pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_binding_pattern_simple() {
    let arena = Arena::new();

    // Test name @ value
    let tokens = create_token_stream(vec![
        TokenType::Identifier("name".to_string()),
        TokenType::At,
        TokenType::Identifier("value".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Binding(binding) => {
                assert_eq!(binding.name, "name");
                match binding.pattern.as_ref() {
                    Pattern::Identifier(name) => assert_eq!(name, "value"),
                    _ => panic!("Expected identifier pattern for binding"),
                }
            }
            _ => panic!("Expected binding pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_complex_or_patterns() {
    let arena = Arena::new();

    // Test 1 | 2 | 3
    let tokens = create_token_stream(vec![
        TokenType::IntegerLiteral(1),
        TokenType::Pipe,
        TokenType::IntegerLiteral(2),
        TokenType::Pipe,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::Or(or_pattern) => {
                assert_eq!(or_pattern.patterns.len(), 3);
            }
            _ => panic!("Expected or pattern, got {:?}", pattern),
        }
    }
}

#[test]
fn test_pattern_parsing_errors() {
    let arena = Arena::new();

    // Test invalid slice pattern: [,]
    let tokens = create_token_stream(vec![
        TokenType::LeftBracket,
        TokenType::Comma,
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    let result = parser.parse_pattern();
    assert!(result.is_err()); // This should be invalid
}

#[test]
fn test_advanced_pattern_integration() {
    let arena = Arena::new();

    // Test that advanced patterns work with existing patterns
    // Test simple range in data class field
    let tokens = create_token_stream(vec![
        TokenType::Identifier("Person".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("age".to_string()),
        TokenType::Colon,
        TokenType::IntegerLiteral(18),
        TokenType::DotDotEqual,
        TokenType::IntegerLiteral(65),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_pattern();
    assert!(result.is_ok());

    if let Ok(pattern) = result {
        match pattern {
            Pattern::DataClass(data_class) => {
                assert_eq!(data_class.name, "Person");
                assert_eq!(data_class.fields.len(), 1);
                assert_eq!(data_class.fields[0].name, "age");
                assert!(data_class.fields[0].pattern.is_some());
            }
            _ => panic!("Expected data class pattern, got {:?}", pattern),
        }
    }
}
