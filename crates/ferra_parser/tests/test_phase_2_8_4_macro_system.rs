//! Tests for Phase 2.8.4: Macro System Foundation
//!
//! This module tests basic macro system functionality including:
//! - Macro invocation parsing (println!("hello"))
//! - Token tree parsing for macro arguments
//! - Basic macro definition parsing framework
//! - Integration with expression parsing

use ferra_parser::{
    ast::{Arena, Expression, GroupDelimiter, TokenTree},
    macro_parser::parser::MacroParser,
    pratt::parser::PrattParser,
    token::{stream::VecTokenStream, TokenType},
};

fn create_token_stream(token_types: Vec<TokenType>) -> VecTokenStream {
    VecTokenStream::from_token_types(token_types)
}

#[test]
fn test_simple_macro_invocation() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::StringLiteral("Hello, world!".to_string()),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_invocation("println".to_string());
    assert!(result.is_ok());

    if let Ok(macro_invocation) = result {
        assert_eq!(macro_invocation.name, "println");
        assert_eq!(macro_invocation.arguments.len(), 1);

        // Check the token group
        if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
            assert!(matches!(group.delimiter, GroupDelimiter::Parentheses));
            assert_eq!(group.tokens.len(), 1);

            if let TokenTree::Token(token) = &group.tokens[0] {
                if let TokenType::StringLiteral(s) = &token.token_type {
                    assert_eq!(s, "Hello, world!");
                }
            }
        }
    }
}

#[test]
fn test_macro_invocation_with_multiple_arguments() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::StringLiteral("Value: {}".to_string()),
        TokenType::Comma,
        TokenType::IntegerLiteral(42),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_invocation("println".to_string());
    assert!(result.is_ok());

    if let Ok(macro_invocation) = result {
        assert_eq!(macro_invocation.name, "println");
        assert_eq!(macro_invocation.arguments.len(), 1);

        if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
            assert_eq!(group.tokens.len(), 3); // string, comma, integer
        }
    }
}

#[test]
fn test_macro_invocation_with_braces() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftBrace,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(10),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_invocation("let_var".to_string());
    assert!(result.is_ok());

    if let Ok(macro_invocation) = result {
        assert_eq!(macro_invocation.name, "let_var");

        if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
            assert!(matches!(group.delimiter, GroupDelimiter::Braces));
            assert_eq!(group.tokens.len(), 3); // identifier, equal, integer
        }
    }
}

#[test]
fn test_macro_invocation_with_brackets() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::IntegerLiteral(2),
        TokenType::Comma,
        TokenType::IntegerLiteral(3),
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_invocation("vec".to_string());
    assert!(result.is_ok());

    if let Ok(macro_invocation) = result {
        assert_eq!(macro_invocation.name, "vec");

        if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
            assert!(matches!(group.delimiter, GroupDelimiter::Brackets));
            assert_eq!(group.tokens.len(), 5); // 1, comma, 2, comma, 3
        }
    }
}

#[test]
fn test_nested_token_groups() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftBrace,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_invocation("calculate".to_string());
    assert!(result.is_ok());

    if let Ok(macro_invocation) = result {
        assert_eq!(macro_invocation.name, "calculate");

        if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
            assert!(matches!(group.delimiter, GroupDelimiter::Braces));
            assert_eq!(group.tokens.len(), 3); // nested group, star, integer

            // Check the nested group
            if let TokenTree::Group(nested_group) = &group.tokens[0] {
                assert!(matches!(
                    nested_group.delimiter,
                    GroupDelimiter::Parentheses
                ));
                assert_eq!(nested_group.tokens.len(), 3); // 1, plus, 2
            }
        }
    }
}

#[test]
fn test_empty_macro_invocation() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_invocation("empty".to_string());
    assert!(result.is_ok());

    if let Ok(macro_invocation) = result {
        assert_eq!(macro_invocation.name, "empty");

        if let TokenTree::Group(group) = &macro_invocation.arguments[0] {
            assert!(matches!(group.delimiter, GroupDelimiter::Parentheses));
            assert_eq!(group.tokens.len(), 0);
        }
    }
}

#[test]
fn test_macro_definition_basic() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::LeftBrace,
        TokenType::Identifier("$x".to_string()),
        TokenType::FatArrow,
        TokenType::Identifier("$x".to_string()),
        TokenType::Plus,
        TokenType::IntegerLiteral(1),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_definition("increment".to_string());
    assert!(result.is_ok());

    if let Ok(macro_def) = result {
        assert_eq!(macro_def.name, "increment");
        assert_eq!(macro_def.rules.len(), 1);

        let rule = &macro_def.rules[0];
        assert_eq!(rule.pattern.len(), 1); // $x
        assert_eq!(rule.replacement.len(), 3); // $x, +, 1
    }
}

#[test]
fn test_macro_definition_multiple_rules() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::LeftBrace,
        TokenType::Identifier("$x".to_string()),
        TokenType::FatArrow,
        TokenType::Identifier("$x".to_string()),
        TokenType::Plus,
        TokenType::IntegerLiteral(1),
        TokenType::Semicolon,
        TokenType::Identifier("$x".to_string()),
        TokenType::Comma,
        TokenType::Identifier("$y".to_string()),
        TokenType::FatArrow,
        TokenType::Identifier("$x".to_string()),
        TokenType::Plus,
        TokenType::Identifier("$y".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);

    let result = parser.parse_macro_definition("add".to_string());
    assert!(result.is_ok());

    if let Ok(macro_def) = result {
        assert_eq!(macro_def.name, "add");
        assert_eq!(macro_def.rules.len(), 2);

        // First rule: $x => $x + 1
        let rule1 = &macro_def.rules[0];
        assert_eq!(rule1.pattern.len(), 1);
        assert_eq!(rule1.replacement.len(), 3);

        // Second rule: $x, $y => $x + $y
        let rule2 = &macro_def.rules[1];
        assert_eq!(rule2.pattern.len(), 3); // $x, comma, $y
        assert_eq!(rule2.replacement.len(), 3); // $x, +, $y
    }
}

#[test]
fn test_macro_in_expression() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Identifier("println".to_string()),
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::StringLiteral("hello".to_string()),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expression) = result {
        match expression {
            Expression::Macro(macro_invocation) => {
                assert_eq!(macro_invocation.name, "println");
                assert_eq!(macro_invocation.arguments.len(), 1);
            }
            _ => panic!("Expected macro invocation expression, got {:?}", expression),
        }
    }
}

#[test]
fn test_macro_in_complex_expression() {
    let arena = Arena::new();
    let tokens = create_token_stream(vec![
        TokenType::Identifier("format".to_string()),
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::StringLiteral("Result: {}".to_string()),
        TokenType::Comma,
        TokenType::IntegerLiteral(42),
        TokenType::RightParen,
        TokenType::Plus,
        TokenType::StringLiteral(" done".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    let result = parser.parse_expression(0);
    assert!(result.is_ok());

    if let Ok(expression) = result {
        match expression {
            Expression::Binary(binary_expr) => {
                // Left side should be macro invocation
                match binary_expr.left.as_ref() {
                    Expression::Macro(macro_invocation) => {
                        assert_eq!(macro_invocation.name, "format");
                    }
                    _ => panic!("Expected macro invocation on left side"),
                }

                // Right side should be string literal
                match binary_expr.right.as_ref() {
                    Expression::Literal(_) => {}
                    _ => panic!("Expected literal on right side"),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expression),
        }
    }
}

#[test]
fn test_macro_parsing_errors() {
    let arena = Arena::new();

    // Test missing closing delimiter
    let tokens = create_token_stream(vec![
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::StringLiteral("hello".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);
    let result = parser.parse_macro_invocation("println".to_string());
    assert!(result.is_err());

    // Test missing bang
    let tokens = create_token_stream(vec![
        TokenType::LeftParen,
        TokenType::StringLiteral("hello".to_string()),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);
    let result = parser.parse_macro_invocation("println".to_string());
    assert!(result.is_err());

    // Test missing fat arrow in macro definition
    let tokens = create_token_stream(vec![
        TokenType::LeftBrace,
        TokenType::Identifier("$x".to_string()),
        TokenType::Identifier("$x".to_string()),
        TokenType::Plus,
        TokenType::IntegerLiteral(1),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = MacroParser::new(&arena, tokens);
    let result = parser.parse_macro_definition("increment".to_string());
    assert!(result.is_err());
}

#[test]
fn test_macro_integration_comprehensive() {
    let arena = Arena::new();

    // Test various macro syntax combinations
    let test_cases = vec![
        (
            "println",
            vec![
                TokenType::Bang,
                TokenType::LeftParen,
                TokenType::StringLiteral("test".to_string()),
                TokenType::RightParen,
            ],
        ),
        (
            "vec",
            vec![
                TokenType::Bang,
                TokenType::LeftBracket,
                TokenType::IntegerLiteral(1),
                TokenType::Comma,
                TokenType::IntegerLiteral(2),
                TokenType::RightBracket,
            ],
        ),
        (
            "block",
            vec![
                TokenType::Bang,
                TokenType::LeftBrace,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(5),
                TokenType::RightBrace,
            ],
        ),
    ];

    for (macro_name, mut token_types) in test_cases {
        token_types.push(TokenType::Eof);
        let tokens = create_token_stream(token_types);
        let mut parser = MacroParser::new(&arena, tokens);

        let result = parser.parse_macro_invocation(macro_name.to_string());
        assert!(result.is_ok(), "Failed to parse macro: {}", macro_name);

        if let Ok(macro_invocation) = result {
            assert_eq!(macro_invocation.name, macro_name);
            assert_eq!(macro_invocation.arguments.len(), 1);
        }
    }
}
