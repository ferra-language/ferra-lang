//! Tests for async function parsing functionality
//!
//! This module contains comprehensive tests for async function declarations,
//! ensuring that the parser correctly handles async function syntax and semantics.

use ferra_parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ferra_parser::{ast, token::VecTokenStream, Arena, TokenType};

    fn create_token_stream(tokens: Vec<TokenType>) -> VecTokenStream {
        VecTokenStream::from_token_types(tokens)
    }

    #[test]
    fn test_basic_async_function() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("test".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Basic async function should parse successfully"
        );

        let compilation_unit = result.unwrap();
        assert_eq!(compilation_unit.items.len(), 1);

        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert_eq!(func.name, "test");
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_async_function_with_parameters() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("fetch_data".to_string()),
            TokenType::LeftParen,
            TokenType::Identifier("url".to_string()),
            TokenType::Colon,
            TokenType::Identifier("String".to_string()),
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Async function with parameters should parse successfully"
        );

        let compilation_unit = result.unwrap();
        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert_eq!(func.name, "fetch_data");
            assert_eq!(func.parameters.len(), 1);
            assert_eq!(func.parameters[0].name, "url");
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_async_function_with_return_type() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("compute".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::Arrow,
            TokenType::Identifier("i32".to_string()),
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Async function with return type should parse successfully"
        );

        let compilation_unit = result.unwrap();
        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert_eq!(func.name, "compute");
            assert!(
                func.return_type.is_some(),
                "Function should have return type"
            );
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_public_async_function() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Pub,
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("api_call".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Public async function should parse successfully"
        );

        let compilation_unit = result.unwrap();
        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert!(func.modifiers.is_public, "Function should be public");
            assert_eq!(func.name, "api_call");
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_unsafe_async_function() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Unsafe,
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("dangerous_async".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Unsafe async function should parse successfully"
        );

        let compilation_unit = result.unwrap();
        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert!(func.modifiers.is_unsafe, "Function should be unsafe");
            assert_eq!(func.name, "dangerous_async");
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_pub_unsafe_async_function() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Pub,
            TokenType::Unsafe,
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("public_dangerous_async".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Public unsafe async function should parse successfully"
        );

        let compilation_unit = result.unwrap();
        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert!(func.modifiers.is_public, "Function should be public");
            assert!(func.modifiers.is_unsafe, "Function should be unsafe");
            assert_eq!(func.name, "public_dangerous_async");
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_multiple_async_functions() {
        let arena = Arena::new();
        let tokens = vec![
            // First async function
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("first".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            // Second async function
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("second".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Multiple async functions should parse successfully"
        );

        let compilation_unit = result.unwrap();
        assert_eq!(compilation_unit.items.len(), 2);

        for item in &compilation_unit.items {
            if let ast::Item::FunctionDecl(func) = item {
                assert!(func.is_async, "All functions should be marked as async");
            } else {
                panic!("Expected function declaration");
            }
        }
    }

    #[test]
    fn test_async_function_with_body() {
        let arena = Arena::new();
        let tokens = vec![
            TokenType::Async,
            TokenType::Fn,
            TokenType::Identifier("with_body".to_string()),
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::Let,
            TokenType::Identifier("x".to_string()),
            TokenType::Equal,
            TokenType::IntegerLiteral(42),
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::Eof,
        ];

        let token_stream = create_token_stream(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        let result = parser.parse_compilation_unit();
        assert!(
            result.is_ok(),
            "Async function with body should parse successfully"
        );

        let compilation_unit = result.unwrap();
        if let ast::Item::FunctionDecl(func) = &compilation_unit.items[0] {
            assert!(func.is_async, "Function should be marked as async");
            assert_eq!(func.name, "with_body");
            // The body should contain statements
            if let Some(body) = &func.body {
                assert!(
                    !body.statements.is_empty(),
                    "Function body should not be empty"
                );
            } else {
                panic!("Function should have a body");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_async_keyword_order_validation() {
        // Test that async must come after modifiers but before fn
        let test_cases = vec![
            // Valid: pub async fn
            (vec![TokenType::Pub, TokenType::Async, TokenType::Fn], true),
            // Valid: unsafe async fn
            (
                vec![TokenType::Unsafe, TokenType::Async, TokenType::Fn],
                true,
            ),
            // Valid: pub unsafe async fn
            (
                vec![
                    TokenType::Pub,
                    TokenType::Unsafe,
                    TokenType::Async,
                    TokenType::Fn,
                ],
                true,
            ),
            // Valid: async fn (no modifiers)
            (vec![TokenType::Async, TokenType::Fn], true),
        ];

        for (prefix_tokens, should_succeed) in test_cases {
            let arena = Arena::new();
            let mut tokens = prefix_tokens;
            tokens.extend(vec![
                TokenType::Identifier("test".to_string()),
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Eof,
            ]);

            let token_stream = create_token_stream(tokens.clone());
            let mut parser = ProgramParser::new(&arena, token_stream);

            let result = parser.parse_compilation_unit();
            if should_succeed {
                assert!(
                    result.is_ok(),
                    "Token sequence {:?} should parse successfully",
                    tokens
                );
            } else {
                assert!(
                    result.is_err(),
                    "Token sequence {:?} should fail to parse",
                    tokens
                );
            }
        }
    }
}
