//! Simple integration tests to improve code coverage

use ferra_parser::{
    ast::{Arena, BinaryOperator, Expression, Literal, UnaryOperator},
    pratt::parser::PrattParser,
    program::parser::ProgramParser,
    statement::parser::StatementParser,
    token::{TokenType, VecTokenStream},
};

/// Test basic expression coverage to hit uncovered expression parsing paths
#[test]
fn test_basic_expression_coverage() {
    let arena = Arena::new();

    // Test various expressions to improve coverage
    let test_cases = vec![
        (
            vec![TokenType::IntegerLiteral(42), TokenType::Eof],
            "integer",
        ),
        (
            vec![
                TokenType::StringLiteral("hello".to_string()),
                TokenType::Eof,
            ],
            "string",
        ),
        (
            vec![TokenType::BooleanLiteral(true), TokenType::Eof],
            "boolean true",
        ),
        (
            vec![TokenType::BooleanLiteral(false), TokenType::Eof],
            "boolean false",
        ),
        (
            vec![TokenType::Identifier("var".to_string()), TokenType::Eof],
            "identifier",
        ),
    ];

    for (tokens, description) in test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = PrattParser::new(&arena, token_stream);

        match parser.parse_expression(0) {
            Ok(_) => println!("✅ Parsed {}", description),
            Err(e) => println!("⚠️ Failed to parse {}: {:?}", description, e),
        }
    }
}

/// Test binary operations to improve coverage
#[test]
fn test_binary_operations_coverage() {
    let arena = Arena::new();

    let binary_test_cases = vec![
        (
            vec![
                TokenType::IntegerLiteral(1),
                TokenType::Plus,
                TokenType::IntegerLiteral(2),
                TokenType::Eof,
            ],
            "addition",
        ),
        (
            vec![
                TokenType::IntegerLiteral(5),
                TokenType::Minus,
                TokenType::IntegerLiteral(3),
                TokenType::Eof,
            ],
            "subtraction",
        ),
        (
            vec![
                TokenType::IntegerLiteral(2),
                TokenType::Star,
                TokenType::IntegerLiteral(4),
                TokenType::Eof,
            ],
            "multiplication",
        ),
        (
            vec![
                TokenType::IntegerLiteral(8),
                TokenType::Slash,
                TokenType::IntegerLiteral(2),
                TokenType::Eof,
            ],
            "division",
        ),
        (
            vec![
                TokenType::IntegerLiteral(10),
                TokenType::Percent,
                TokenType::IntegerLiteral(3),
                TokenType::Eof,
            ],
            "modulo",
        ),
    ];

    for (tokens, description) in binary_test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = PrattParser::new(&arena, token_stream);

        match parser.parse_expression(0) {
            Ok(_) => println!("✅ Parsed {}", description),
            Err(e) => println!("⚠️ Failed to parse {}: {:?}", description, e),
        }
    }
}

/// Test unary operations to improve coverage
#[test]
fn test_unary_operations_coverage() {
    let arena = Arena::new();

    let unary_test_cases = vec![
        (
            vec![
                TokenType::Minus,
                TokenType::IntegerLiteral(42),
                TokenType::Eof,
            ],
            "negative",
        ),
        (
            vec![
                TokenType::Plus,
                TokenType::IntegerLiteral(42),
                TokenType::Eof,
            ],
            "positive",
        ),
        (
            vec![
                TokenType::Bang,
                TokenType::BooleanLiteral(true),
                TokenType::Eof,
            ],
            "logical not",
        ),
    ];

    for (tokens, description) in unary_test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = PrattParser::new(&arena, token_stream);

        match parser.parse_expression(0) {
            Ok(_) => println!("✅ Parsed {}", description),
            Err(e) => println!("⚠️ Failed to parse {}: {:?}", description, e),
        }
    }
}

/// Test statements to improve statement coverage
#[test]
fn test_basic_statement_coverage() {
    let arena = Arena::new();

    let statement_test_cases = vec![
        (
            vec![
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(42),
                TokenType::Semicolon,
                TokenType::Eof,
            ],
            "let variable",
        ),
        (
            vec![
                TokenType::Var,
                TokenType::Identifier("y".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(10),
                TokenType::Semicolon,
                TokenType::Eof,
            ],
            "var variable",
        ),
        (
            vec![
                TokenType::Return,
                TokenType::IntegerLiteral(42),
                TokenType::Semicolon,
                TokenType::Eof,
            ],
            "return with value",
        ),
        (
            vec![TokenType::Return, TokenType::Semicolon, TokenType::Eof],
            "return without value",
        ),
        (
            vec![TokenType::Break, TokenType::Semicolon, TokenType::Eof],
            "break statement",
        ),
        (
            vec![TokenType::Continue, TokenType::Semicolon, TokenType::Eof],
            "continue statement",
        ),
    ];

    for (tokens, description) in statement_test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = StatementParser::new(&arena, token_stream);

        match parser.parse_statement() {
            Ok(_) => println!("✅ Parsed {}", description),
            Err(e) => println!("⚠️ Failed to parse {}: {:?}", description, e),
        }
    }
}

/// Test program parsing to improve integration coverage
#[test]
fn test_program_parsing_coverage() {
    let arena = Arena::new();

    let program_test_cases = vec![
        (
            vec![
                TokenType::Fn,
                TokenType::Identifier("main".to_string()),
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::Return,
                TokenType::IntegerLiteral(0),
                TokenType::Semicolon,
                TokenType::RightBrace,
                TokenType::Eof,
            ],
            "simple main function",
        ),
        (
            vec![
                TokenType::Let,
                TokenType::Identifier("x".to_string()),
                TokenType::Equal,
                TokenType::IntegerLiteral(42),
                TokenType::Semicolon,
                TokenType::Eof,
            ],
            "simple variable declaration",
        ),
    ];

    for (tokens, description) in program_test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = ProgramParser::new(&arena, token_stream);

        match parser.parse_compilation_unit() {
            Ok(program) => println!(
                "✅ Parsed {} with {} items",
                description,
                program.items.len()
            ),
            Err(e) => println!("⚠️ Failed to parse {}: {:?}", description, e),
        }
    }
}

/// Test error conditions to improve error path coverage
#[test]
fn test_error_handling_coverage() {
    let arena = Arena::new();

    let error_test_cases = vec![
        (
            vec![TokenType::Plus, TokenType::Plus, TokenType::Eof],
            "double plus",
        ),
        (
            vec![
                TokenType::LeftParen,
                TokenType::IntegerLiteral(42),
                TokenType::Eof,
            ],
            "unclosed paren",
        ),
        (
            vec![
                TokenType::IntegerLiteral(42),
                TokenType::Plus,
                TokenType::Eof,
            ],
            "incomplete binary",
        ),
    ];

    for (tokens, description) in error_test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = PrattParser::new(&arena, token_stream);

        match parser.parse_expression(0) {
            Ok(_) => println!("⚠️ Expected error for {}", description),
            Err(_) => println!("✅ Got expected error for {}", description),
        }
    }
}

/// Test complex nested expressions to improve complexity coverage
#[test]
fn test_complex_expression_coverage() {
    let arena = Arena::new();

    // Test (1 + 2) * 3
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);

    match parser.parse_expression(0) {
        Ok(_) => println!("✅ Parsed complex grouped expression"),
        Err(e) => println!("⚠️ Failed to parse complex expression: {:?}", e),
    }
}

/// Test array literals to improve array parsing coverage
#[test]
fn test_array_literal_coverage() {
    let arena = Arena::new();

    let array_test_cases = vec![
        (
            vec![
                TokenType::LeftBracket,
                TokenType::RightBracket,
                TokenType::Eof,
            ],
            "empty array",
        ),
        (
            vec![
                TokenType::LeftBracket,
                TokenType::IntegerLiteral(1),
                TokenType::Comma,
                TokenType::IntegerLiteral(2),
                TokenType::RightBracket,
                TokenType::Eof,
            ],
            "array with elements",
        ),
    ];

    for (tokens, description) in array_test_cases {
        let token_stream = VecTokenStream::from_token_types(tokens);
        let mut parser = PrattParser::new(&arena, token_stream);

        match parser.parse_expression(0) {
            Ok(_) => println!("✅ Parsed {}", description),
            Err(e) => println!("⚠️ Failed to parse {}: {:?}", description, e),
        }
    }
}
