//! Phase 2.4: Block and Scope Parsing Tests
//!
//! Comprehensive test suite for advanced block parsing features

use ferra_parser::{
    ast::{Arena, BinaryOperator, Expression, Statement, UnaryOperator},
    block::parser::{BlockParser, BlockStyle, ScopeInfo},
    error::ParseError,
    token::{Span, Token, TokenType, VecTokenStream},
};

fn create_test_span() -> Span {
    Span::new(0, 10, 1, 1)
}

/// Test basic braced block parsing
#[test]
fn test_simple_braced_block() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(TokenType::Identifier("x".to_string()), create_test_span()),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(42), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_braced_block(&mut stream);
    match &result {
        Ok(block) => {
            println!("Success! Block has {} statements", block.statements.len());
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
    assert!(result.is_ok());

    let block = result.unwrap();
    assert!(block.is_braced);
    assert_eq!(block.scope_depth, 0);
    assert!(!block.is_unsafe);
    assert!(!block.is_async);
    assert!(!block.is_try);
    assert_eq!(block.statements.len(), 1);
}

/// Test indented block parsing
#[test]
fn test_simple_indented_block() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::Colon, create_test_span()),
        Token::new(TokenType::Newline, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(TokenType::Identifier("x".to_string()), create_test_span()),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(42), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_indented_block(&mut stream);
    assert!(result.is_ok());

    let block = result.unwrap();
    assert!(!block.is_braced);
    assert_eq!(block.scope_depth, 0);
    assert_eq!(block.statements.len(), 1);
}

/// Test mixed block style error detection
#[test]
fn test_mixed_block_styles_error() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(TokenType::Identifier("x".to_string()), create_test_span()),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(42), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    // First parse a braced block to set the style
    let _first_block = parser.parse_braced_block(&mut stream).unwrap();

    // Now try to parse an indented block - should fail
    let indented_tokens = vec![
        Token::new(TokenType::Colon, create_test_span()),
        Token::new(TokenType::Newline, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut indented_stream = VecTokenStream::new(indented_tokens);

    let result = parser.parse_indented_block(&mut indented_stream);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ParseError::MixedBlockStyles { .. }
    ));
}

/// Test unsafe block parsing
#[test]
fn test_unsafe_block() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::Unsafe, create_test_span()),
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(TokenType::Identifier("ptr".to_string()), create_test_span()),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(0), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_unsafe_block(&mut stream);
    assert!(result.is_ok());

    let block = result.unwrap();
    assert!(block.is_braced);
    assert!(block.is_unsafe);
    assert!(!block.is_async);
    assert!(!block.is_try);
    assert_eq!(block.statements.len(), 1);
}

/// Test async block parsing
#[test]
fn test_async_block() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::Async, create_test_span()),
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(
            TokenType::Identifier("result".to_string()),
            create_test_span(),
        ),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(42), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_async_block(&mut stream);
    assert!(result.is_ok());

    let block = result.unwrap();
    assert!(block.is_braced);
    assert!(!block.is_unsafe);
    assert!(block.is_async);
    assert!(!block.is_try);
    assert_eq!(block.statements.len(), 1);
}

/// Test labeled block parsing
#[test]
fn test_labeled_block() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(TokenType::Identifier("x".to_string()), create_test_span()),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(42), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_labeled_block(&mut stream, "outer".to_string());
    assert!(result.is_ok());

    let block = result.unwrap();
    assert!(block.is_braced);
    assert_eq!(block.label, Some("outer".to_string()));
    assert_eq!(block.statements.len(), 1);
}

/// Test nested blocks with scope depth tracking
#[test]
fn test_nested_blocks_scope_depth() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::Let, create_test_span()),
        Token::new(
            TokenType::Identifier("inner".to_string()),
            create_test_span(),
        ),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(1), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_braced_block(&mut stream);
    assert!(result.is_ok());

    let outer_block = result.unwrap();
    assert_eq!(outer_block.scope_depth, 0);
    assert_eq!(outer_block.statements.len(), 1);
}

/// Test scope validation
#[test]
fn test_scope_validation() {
    let arena = Arena::new();
    let parser = BlockParser::new(&arena);

    // Test valid scope
    let valid_scope = ScopeInfo {
        depth: 1,
        variables: vec!["x".to_string(), "y".to_string()],
        is_unsafe: false,
        is_async: false,
        label: None,
    };

    assert!(parser.validate_scope(&valid_scope).is_ok());

    // Test invalid scope with duplicate variables
    let invalid_scope = ScopeInfo {
        depth: 1,
        variables: vec!["x".to_string(), "y".to_string(), "x".to_string()],
        is_unsafe: false,
        is_async: false,
        label: None,
    };

    let result = parser.validate_scope(&invalid_scope);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ParseError::VariableRedefinition { .. }
    ));
}

/// Test block style consistency
#[test]
fn test_block_style_consistency() {
    assert_eq!(BlockStyle::Braced, BlockStyle::Braced);
    assert_eq!(BlockStyle::Indented, BlockStyle::Indented);
    assert_ne!(BlockStyle::Braced, BlockStyle::Indented);
}

/// Test automatic block style detection
#[test]
fn test_automatic_block_detection() {
    let arena = Arena::new();
    let mut parser = BlockParser::new(&arena);

    // Test detection of braced block
    let braced_tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut braced_stream = VecTokenStream::new(braced_tokens);

    let result = parser.parse_block(&mut braced_stream);
    assert!(result.is_ok());
    assert!(result.unwrap().is_braced);

    // Test detection of indented block
    let indented_tokens = vec![
        Token::new(TokenType::Colon, create_test_span()),
        Token::new(TokenType::Newline, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut indented_stream = VecTokenStream::new(indented_tokens);
    let mut new_parser = BlockParser::new(&arena); // Need fresh parser for different style

    let result = new_parser.parse_block(&mut indented_stream);
    assert!(result.is_ok());
    assert!(!result.unwrap().is_braced);
}

/// Test block with complex statements
#[test]
fn test_complex_block_parsing() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        // Variable declaration
        Token::new(TokenType::Let, create_test_span()),
        Token::new(TokenType::Identifier("x".to_string()), create_test_span()),
        Token::new(TokenType::Equal, create_test_span()),
        Token::new(TokenType::IntegerLiteral(42), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        // Expression statement
        Token::new(TokenType::Identifier("y".to_string()), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        // Return statement
        Token::new(TokenType::Return, create_test_span()),
        Token::new(TokenType::Identifier("x".to_string()), create_test_span()),
        Token::new(TokenType::Semicolon, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_braced_block(&mut stream);
    match &result {
        Ok(block) => {
            println!("Success! Block has {} statements", block.statements.len());
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
    assert!(result.is_ok());

    let block = result.unwrap();
    assert_eq!(block.statements.len(), 3); // let, expression, return
    assert!(block.is_braced);
}

/// Test error handling for invalid block syntax
#[test]
fn test_invalid_block_syntax() {
    let arena = Arena::new();
    let tokens = vec![
        Token::new(TokenType::IntegerLiteral(42), create_test_span()), // Not a block start
        Token::new(TokenType::Eof, create_test_span()),
    ];

    let mut stream = VecTokenStream::new(tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_block(&mut stream);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ParseError::ExpectedBlock { .. }
    ));
}

/// Test empty blocks
#[test]
fn test_empty_blocks() {
    let arena = Arena::new();

    // Empty braced block
    let braced_tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut braced_stream = VecTokenStream::new(braced_tokens);
    let mut parser = BlockParser::new(&arena);

    let result = parser.parse_braced_block(&mut braced_stream);
    assert!(result.is_ok());

    let block = result.unwrap();
    assert_eq!(block.statements.len(), 0);
    assert!(block.is_braced);

    // Empty indented block
    let indented_tokens = vec![
        Token::new(TokenType::Colon, create_test_span()),
        Token::new(TokenType::Newline, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut indented_stream = VecTokenStream::new(indented_tokens);
    let mut new_parser = BlockParser::new(&arena);

    let result = new_parser.parse_indented_block(&mut indented_stream);
    assert!(result.is_ok());

    let block = result.unwrap();
    assert_eq!(block.statements.len(), 0);
    assert!(!block.is_braced);
}

/// Test convenience functions
#[test]
fn test_convenience_functions() {
    let arena = Arena::new();

    // Test parse_braced_block convenience function
    let tokens = vec![
        Token::new(TokenType::LeftBrace, create_test_span()),
        Token::new(TokenType::RightBrace, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut stream = VecTokenStream::new(tokens);

    let result = ferra_parser::block::parser::parse_braced_block(&arena, &mut stream);
    assert!(result.is_ok());

    // Test parse_indented_block convenience function
    let indented_tokens = vec![
        Token::new(TokenType::Colon, create_test_span()),
        Token::new(TokenType::Newline, create_test_span()),
        Token::new(TokenType::Eof, create_test_span()),
    ];
    let mut indented_stream = VecTokenStream::new(indented_tokens);

    let result = ferra_parser::block::parser::parse_indented_block(&arena, &mut indented_stream);
    assert!(result.is_ok());
}

#[test]
fn test_complex_expressions_in_blocks() {
    let arena = Arena::new();

    // Test complex binary expressions
    let mut parser = BlockParser::new(&arena);
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("result".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let result = parser.parse_braced_block(&mut tokens);

    assert!(
        result.is_ok(),
        "Failed to parse complex binary expression: {:?}",
        result.err()
    );
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 1);

    // Verify the complex expression was parsed correctly
    match &block.statements[0] {
        Statement::VariableDecl(var_decl) => {
            assert_eq!(var_decl.name, "result");
            assert!(var_decl.initializer.is_some());
            // Should parse as 1 + (2 * 3) due to precedence
            match var_decl.initializer.as_ref().unwrap() {
                Expression::Binary(binary) => {
                    assert!(matches!(binary.operator, BinaryOperator::Add));
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_function_calls_in_blocks() {
    let arena = Arena::new();

    // Test function call expressions
    let mut parser = BlockParser::new(&arena);
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBrace,
        TokenType::Identifier("println".to_string()),
        TokenType::LeftParen,
        TokenType::StringLiteral("Hello, world!".to_string()),
        TokenType::RightParen,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let result = parser.parse_braced_block(&mut tokens);

    assert!(
        result.is_ok(),
        "Failed to parse function call: {:?}",
        result.err()
    );
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 1);

    // Verify the function call was parsed correctly
    match &block.statements[0] {
        Statement::Expression(expr) => match expr {
            Expression::Call(call) => {
                match call.callee.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "println"),
                    _ => panic!("Expected function name to be identifier"),
                }
                assert_eq!(call.arguments.len(), 1);
            }
            _ => panic!("Expected function call expression"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_member_access_in_blocks() {
    let arena = Arena::new();

    // Test member access expressions
    let mut parser = BlockParser::new(&arena);
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("value".to_string()),
        TokenType::Equal,
        TokenType::Identifier("object".to_string()),
        TokenType::Dot,
        TokenType::Identifier("property".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let result = parser.parse_braced_block(&mut tokens);

    assert!(
        result.is_ok(),
        "Failed to parse member access: {:?}",
        result.err()
    );
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 1);

    // Verify the member access was parsed correctly
    match &block.statements[0] {
        Statement::VariableDecl(var_decl) => {
            assert_eq!(var_decl.name, "value");
            assert!(var_decl.initializer.is_some());
            match var_decl.initializer.as_ref().unwrap() {
                Expression::MemberAccess(access) => {
                    match access.object.as_ref() {
                        Expression::Identifier(name) => assert_eq!(name, "object"),
                        _ => panic!("Expected object to be identifier"),
                    }
                    assert_eq!(access.member, "property");
                }
                _ => panic!("Expected member access expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_array_literals_in_blocks() {
    let arena = Arena::new();

    // Test array literal expressions
    let mut parser = BlockParser::new(&arena);
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("numbers".to_string()),
        TokenType::Equal,
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::IntegerLiteral(2),
        TokenType::Comma,
        TokenType::IntegerLiteral(3),
        TokenType::RightBracket,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let result = parser.parse_braced_block(&mut tokens);

    assert!(
        result.is_ok(),
        "Failed to parse array literal: {:?}",
        result.err()
    );
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 1);

    // Verify the array literal was parsed correctly
    match &block.statements[0] {
        Statement::VariableDecl(var_decl) => {
            assert_eq!(var_decl.name, "numbers");
            assert!(var_decl.initializer.is_some());
            match var_decl.initializer.as_ref().unwrap() {
                Expression::Array(array) => {
                    assert_eq!(array.elements.len(), 3);
                }
                _ => panic!("Expected array literal expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_complex_nested_expressions_in_blocks() {
    let arena = Arena::new();

    // Test deeply nested expressions: obj.method(array[index + 1])
    let mut parser = BlockParser::new(&arena);
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBrace,
        TokenType::Identifier("obj".to_string()),
        TokenType::Dot,
        TokenType::Identifier("method".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("array".to_string()),
        TokenType::LeftBracket,
        TokenType::Identifier("index".to_string()),
        TokenType::Plus,
        TokenType::IntegerLiteral(1),
        TokenType::RightBracket,
        TokenType::RightParen,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let result = parser.parse_braced_block(&mut tokens);

    assert!(
        result.is_ok(),
        "Failed to parse complex nested expression: {:?}",
        result.err()
    );
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 1);

    // Verify the complex expression was parsed correctly
    match &block.statements[0] {
        Statement::Expression(expr) => {
            match expr {
                Expression::Call(call) => {
                    // Should be a member access for obj.method
                    match call.callee.as_ref() {
                        Expression::MemberAccess(access) => {
                            assert_eq!(access.member, "method");
                        }
                        _ => panic!("Expected member access for function"),
                    }
                    // Should have one argument: array[index + 1]
                    assert_eq!(call.arguments.len(), 1);
                    match &call.arguments[0] {
                        Expression::Index(index) => {
                            match index.object.as_ref() {
                                Expression::Identifier(name) => assert_eq!(name, "array"),
                                _ => panic!("Expected array identifier"),
                            }
                            // Index should be a binary expression (index + 1)
                            match index.index.as_ref() {
                                Expression::Binary(binary) => {
                                    assert!(matches!(binary.operator, BinaryOperator::Add));
                                }
                                _ => panic!("Expected binary expression for index"),
                            }
                        }
                        _ => panic!("Expected index access as argument"),
                    }
                }
                _ => panic!("Expected function call expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_unary_expressions_in_blocks() {
    let arena = Arena::new();

    // Test unary expressions
    let mut parser = BlockParser::new(&arena);
    let mut tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("negated".to_string()),
        TokenType::Equal,
        TokenType::Minus,
        TokenType::Identifier("value".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);

    let result = parser.parse_braced_block(&mut tokens);

    assert!(
        result.is_ok(),
        "Failed to parse unary expression: {:?}",
        result.err()
    );
    let block = result.unwrap();
    assert_eq!(block.statements.len(), 1);

    // Verify the unary expression was parsed correctly
    match &block.statements[0] {
        Statement::VariableDecl(var_decl) => {
            assert_eq!(var_decl.name, "negated");
            assert!(var_decl.initializer.is_some());
            match var_decl.initializer.as_ref().unwrap() {
                Expression::Unary(unary) => {
                    assert!(matches!(unary.operator, UnaryOperator::Minus));
                    match unary.operand.as_ref() {
                        Expression::Identifier(name) => assert_eq!(name, "value"),
                        _ => panic!("Expected identifier operand"),
                    }
                }
                _ => panic!("Expected unary expression"),
            }
        }
        _ => panic!("Expected variable declaration"),
    }
}
