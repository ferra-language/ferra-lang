//! Phase 2.6: Integration Testing
//!
//! Comprehensive integration tests that verify the interaction between
//! all parser components and test complex real-world scenarios.

use ferra_parser::{
    ast::{Arena, Item},
    token::{TokenType, VecTokenStream},
    ProgramParser,
};

#[test]
fn test_complex_program_with_all_features() {
    // Test a complex program that uses all major language features
    let tokens = vec![
        // Data class: data Vector3 { x: float, y: float, z: float }
        TokenType::Data,
        TokenType::Identifier("Vector3".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("float".to_string()),
        TokenType::Comma,
        TokenType::Identifier("y".to_string()),
        TokenType::Colon,
        TokenType::Identifier("float".to_string()),
        TokenType::Comma,
        TokenType::Identifier("z".to_string()),
        TokenType::Colon,
        TokenType::Identifier("float".to_string()),
        TokenType::RightBrace,
        // Extern block: extern "C" { fn sqrt(x: float) -> float; }
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()),
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("sqrt".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("float".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("float".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        // Function: fn magnitude(v: Vector3) -> float { let result = 0.0; }
        TokenType::Fn,
        TokenType::Identifier("magnitude".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("v".to_string()),
        TokenType::Colon,
        TokenType::Identifier("Vector3".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("float".to_string()),
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("result".to_string()),
        TokenType::Equal,
        TokenType::FloatLiteral(0.0),
        TokenType::Semicolon,
        TokenType::RightBrace,
        // Main function: fn main() -> int { let x = 0; }
        TokenType::Fn,
        TokenType::Identifier("main".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("x".to_string()),
        TokenType::Equal,
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
        "Failed to parse complex program: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 4, "Expected 4 items in compilation unit");

    // Verify all items are parsed correctly
    match &unit.items[0] {
        Item::DataClassDecl(data) => {
            assert_eq!(data.name, "Vector3");
            assert_eq!(data.fields.len(), 3);
        }
        _ => panic!("Expected first item to be data class"),
    }

    match &unit.items[1] {
        Item::ExternBlock(extern_block) => {
            assert_eq!(extern_block.abi, "C");
            assert_eq!(extern_block.items.len(), 1);
        }
        _ => panic!("Expected second item to be extern block"),
    }

    match &unit.items[2] {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "magnitude");
            assert_eq!(func.parameters.len(), 1);
            assert!(func.return_type.is_some());
        }
        _ => panic!("Expected third item to be function"),
    }

    match &unit.items[3] {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "main");
            assert_eq!(func.parameters.len(), 0);
        }
        _ => panic!("Expected fourth item to be function"),
    }
}

#[test]
fn test_multiple_data_classes() {
    // Test parsing multiple data classes with different field counts
    let tokens = vec![
        // data Point { x: int, y: int }
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
        // data Color { r: int, g: int, b: int, a: int }
        TokenType::Data,
        TokenType::Identifier("Color".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("r".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("g".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("b".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("a".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBrace,
        // data Empty { }
        TokenType::Data,
        TokenType::Identifier("Empty".to_string()),
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    assert!(
        result.is_ok(),
        "Failed to parse multiple data classes: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 3);

    // Verify field counts
    match &unit.items[0] {
        Item::DataClassDecl(data) => {
            assert_eq!(data.name, "Point");
            assert_eq!(data.fields.len(), 2);
        }
        _ => panic!("Expected data class"),
    }

    match &unit.items[1] {
        Item::DataClassDecl(data) => {
            assert_eq!(data.name, "Color");
            assert_eq!(data.fields.len(), 4);
        }
        _ => panic!("Expected data class"),
    }

    match &unit.items[2] {
        Item::DataClassDecl(data) => {
            assert_eq!(data.name, "Empty");
            assert_eq!(data.fields.len(), 0);
        }
        _ => panic!("Expected data class"),
    }
}

#[test]
fn test_functions_with_different_signatures() {
    // Test functions with various parameter counts and return types
    let tokens = vec![
        // fn no_params() { }
        TokenType::Fn,
        TokenType::Identifier("no_params".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        // fn one_param(x: int) -> int { let result = x; }
        TokenType::Fn,
        TokenType::Identifier("one_param".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("x".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("result".to_string()),
        TokenType::Equal,
        TokenType::Identifier("x".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        // fn three_params(a: int, b: float, c: string) -> bool { let flag = true; }
        TokenType::Fn,
        TokenType::Identifier("three_params".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("a".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::Comma,
        TokenType::Identifier("b".to_string()),
        TokenType::Colon,
        TokenType::Identifier("float".to_string()),
        TokenType::Comma,
        TokenType::Identifier("c".to_string()),
        TokenType::Colon,
        TokenType::Identifier("string".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("bool".to_string()),
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("flag".to_string()),
        TokenType::Equal,
        TokenType::BooleanLiteral(true),
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
        "Failed to parse functions with different signatures: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 3);

    // Verify function signatures
    match &unit.items[0] {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "no_params");
            assert_eq!(func.parameters.len(), 0);
            assert!(func.return_type.is_none());
        }
        _ => panic!("Expected function"),
    }

    match &unit.items[1] {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "one_param");
            assert_eq!(func.parameters.len(), 1);
            assert!(func.return_type.is_some());
        }
        _ => panic!("Expected function"),
    }

    match &unit.items[2] {
        Item::FunctionDecl(func) => {
            assert_eq!(func.name, "three_params");
            assert_eq!(func.parameters.len(), 3);
            assert!(func.return_type.is_some());
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_extern_blocks_with_multiple_items() {
    // Test extern blocks with multiple functions and variables
    let tokens = vec![
        // extern "C" { fn malloc(size: int) -> ptr; fn free(ptr: ptr); static errno: int; }
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()),
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("malloc".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("size".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("ptr".to_string()),
        TokenType::Semicolon,
        TokenType::Fn,
        TokenType::Identifier("free".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("ptr".to_string()),
        TokenType::Colon,
        TokenType::Identifier("ptr".to_string()),
        TokenType::RightParen,
        TokenType::Semicolon,
        TokenType::Static,
        TokenType::Identifier("errno".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
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
        "Failed to parse extern block with multiple items: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 1);

    match &unit.items[0] {
        Item::ExternBlock(extern_block) => {
            assert_eq!(extern_block.abi, "C");
            assert_eq!(extern_block.items.len(), 3);

            // Verify extern items
            match &extern_block.items[0] {
                ferra_parser::ast::ExternItem::Function(func) => {
                    assert_eq!(func.name, "malloc");
                    assert_eq!(func.parameters.len(), 1);
                    assert!(func.return_type.is_some());
                }
                _ => panic!("Expected extern function"),
            }

            match &extern_block.items[1] {
                ferra_parser::ast::ExternItem::Function(func) => {
                    assert_eq!(func.name, "free");
                    assert_eq!(func.parameters.len(), 1);
                    assert!(func.return_type.is_none());
                }
                _ => panic!("Expected extern function"),
            }

            match &extern_block.items[2] {
                ferra_parser::ast::ExternItem::Variable(var) => {
                    assert_eq!(var.name, "errno");
                }
                _ => panic!("Expected extern variable"),
            }
        }
        _ => panic!("Expected extern block"),
    }
}

#[test]
fn test_mixed_top_level_items() {
    // Test a program with mixed top-level items in various orders
    let tokens = vec![
        // fn first_function() { }
        TokenType::Fn,
        TokenType::Identifier("first_function".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        // data MyData { value: int }
        TokenType::Data,
        TokenType::Identifier("MyData".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("value".to_string()),
        TokenType::Colon,
        TokenType::Identifier("int".to_string()),
        TokenType::RightBrace,
        // extern "C" { fn external_func(); }
        TokenType::Extern,
        TokenType::StringLiteral("C".to_string()),
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("external_func".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Semicolon,
        TokenType::RightBrace,
        // fn second_function(data: MyData) -> int { let result = 42; }
        TokenType::Fn,
        TokenType::Identifier("second_function".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("data".to_string()),
        TokenType::Colon,
        TokenType::Identifier("MyData".to_string()),
        TokenType::RightParen,
        TokenType::Arrow,
        TokenType::Identifier("int".to_string()),
        TokenType::LeftBrace,
        TokenType::Let,
        TokenType::Identifier("result".to_string()),
        TokenType::Equal,
        TokenType::IntegerLiteral(42),
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
        "Failed to parse mixed top-level items: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 4);

    // Verify order and types
    assert!(matches!(unit.items[0], Item::FunctionDecl(_)));
    assert!(matches!(unit.items[1], Item::DataClassDecl(_)));
    assert!(matches!(unit.items[2], Item::ExternBlock(_)));
    assert!(matches!(unit.items[3], Item::FunctionDecl(_)));
}

#[test]
fn test_error_recovery_in_program_parsing() {
    // Test that the parser can recover from errors and continue parsing
    let tokens = vec![
        // Valid function
        TokenType::Fn,
        TokenType::Identifier("valid_func".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        // Invalid token that should cause error
        TokenType::Plus, // This should cause an error
        // Another valid function after error
        TokenType::Fn,
        TokenType::Identifier("another_func".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    // Should have errors but still parse some items
    assert!(result.is_err(), "Expected parsing errors");
    assert!(
        parser.has_errors(),
        "Expected error collection to have errors"
    );

    let errors = parser.get_errors();
    assert!(!errors.is_empty(), "Expected at least one error");
}

#[test]
fn test_empty_program() {
    // Test parsing an empty program (just EOF)
    let tokens = vec![TokenType::Eof];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let result = parser.parse_compilation_unit();

    assert!(
        result.is_ok(),
        "Failed to parse empty program: {:?}",
        result.err()
    );
    let unit = result.unwrap();
    assert_eq!(unit.items.len(), 0, "Expected empty compilation unit");
}

#[test]
fn test_program_with_diagnostics() {
    // Test the diagnostic reporting functionality
    let tokens = vec![
        TokenType::Fn,
        TokenType::Identifier("test".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::Eof,
    ];

    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    let (program, report) = parser.parse_program_with_diagnostics();

    assert!(program.is_some(), "Expected successful parsing");
    assert!(!report.has_errors(), "Expected no errors in report");

    let unit = program.unwrap();
    assert_eq!(unit.items.len(), 1, "Expected one function");
}
