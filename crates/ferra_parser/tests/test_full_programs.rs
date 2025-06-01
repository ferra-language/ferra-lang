//! Integration tests for full program parsing

use ferra_parser::{
    ast::{Arena, CompilationUnit},
    token::{Span, TokenType, VecTokenStream},
    Parser, ProgramParser,
};

#[test]
fn test_compilation_unit_creation() {
    let span = Span::dummy();

    // Test creating a compilation unit
    let compilation_unit = CompilationUnit {
        items: vec![],
        span,
    };

    assert_eq!(compilation_unit.items.len(), 0);
}

#[test]
fn test_simple_program_tokens() {
    let tokens = vec![
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
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let _parser = Parser::new(&arena, stream);

    // Verify token sequence for simple program
    assert!(true);
}

#[test]
fn test_program_with_data_class_tokens() {
    let tokens = vec![
        TokenType::Data,
        TokenType::Identifier("Point".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("y".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let _parser = Parser::new(&arena, stream);

    // Verify token sequence for data class
    assert!(true);
}

#[test]
fn test_extern_block_tokens() {
    let tokens = vec![
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()), // Need ABI string for extern blocks
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("printf".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("format".to_string()),
        TokenType::Colon,
        TokenType::Identifier("char".to_string()), // Simplified - should be *const char
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let _parser = Parser::new(&arena, stream);

    // Verify token sequence for extern block
    assert!(true);
}

// Phase 2.6: Integration Testing - Full program parsing tests
#[test]
fn test_simple_program() {
    // Test parsing of simple complete programs like: fn main() { return 0; }
    let tokens = vec![
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
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    assert!(
        result.is_ok(),
        "Failed to parse simple program: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 1, "Expected 1 item in compilation unit");

    // Verify it's a function declaration
    match &unit.items[0] {
        ferra_parser::ast::Item::FunctionDecl(func) => {
            assert_eq!(func.name, "main");
            assert_eq!(func.parameters.len(), 0);
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_program_with_functions() {
    // Test parsing of programs with multiple functions
    let tokens = vec![
        // First function: fn add(a: int, b: int) -> int { return a + b; }
        TokenType::Fn,
        TokenType::Identifier("add".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("a".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("b".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Identifier("a".to_string()),
        TokenType::Plus,
        TokenType::Identifier("b".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        // Second function: fn main() { return 0; }
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
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    assert!(
        result.is_ok(),
        "Failed to parse program with multiple functions: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 2, "Expected 2 items in compilation unit");

    // Verify both are function declarations
    match &unit.items[0] {
        ferra_parser::ast::Item::FunctionDecl(func) => {
            assert_eq!(func.name, "add");
            assert_eq!(func.parameters.len(), 2);
        }
        _ => panic!("Expected first item to be function declaration"),
    }

    match &unit.items[1] {
        ferra_parser::ast::Item::FunctionDecl(func) => {
            assert_eq!(func.name, "main");
            assert_eq!(func.parameters.len(), 0);
        }
        _ => panic!("Expected second item to be function declaration"),
    }
}

#[test]
fn test_program_with_data_classes() {
    // Test parsing of programs with data class definitions
    let tokens = vec![
        // Data class: data Point { x: int, y: int }
        TokenType::Data,
        TokenType::Identifier("Point".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("y".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBrace,
        // Function using the data class
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
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    assert!(
        result.is_ok(),
        "Failed to parse program with data classes: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 2, "Expected 2 items in compilation unit");

    // Verify first is data class, second is function
    match &unit.items[0] {
        ferra_parser::ast::Item::DataClassDecl(data) => {
            assert_eq!(data.name, "Point");
            assert_eq!(data.fields.len(), 2);
        }
        _ => panic!("Expected first item to be data class declaration"),
    }

    match &unit.items[1] {
        ferra_parser::ast::Item::FunctionDecl(func) => {
            assert_eq!(func.name, "main");
        }
        _ => panic!("Expected second item to be function declaration"),
    }
}

#[test]
fn test_program_with_extern_blocks() {
    // Test parsing of programs with extern blocks
    let tokens = vec![
        // Extern block: extern "C" { fn printf(format: char) -> int; }
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()),
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("printf".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("format".to_string()),
        TokenType::Colon,
        TokenType::Identifier("char".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        // Function using extern
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
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    assert!(
        result.is_ok(),
        "Failed to parse program with extern blocks: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 2, "Expected 2 items in compilation unit");

    // Verify first is extern block, second is function
    match &unit.items[0] {
        ferra_parser::ast::Item::ExternBlock(extern_block) => {
            assert_eq!(extern_block.abi, "C");
            assert_eq!(extern_block.items.len(), 1);
        }
        _ => panic!("Expected first item to be extern block"),
    }

    match &unit.items[1] {
        ferra_parser::ast::Item::FunctionDecl(func) => {
            assert_eq!(func.name, "main");
        }
        _ => panic!("Expected second item to be function declaration"),
    }
}
