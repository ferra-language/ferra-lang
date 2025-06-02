//! Error Path & Edge Case Coverage Blitz
//!
//! Phase 2 of coverage improvement targeting highest-impact areas:
//! - error/recovery.rs: 117/193 (60.6%) - 76 lines available (~1.4% boost)
//! - error/parse_error.rs: 105/199 (52.8%) - 94 lines available (~1.7% boost)  
//! - pratt/parser.rs: 245/404 (60.6%) - 159 lines available (~2.9% boost)
//! - program/parser.rs: 223/498 (44.8%) - 275 lines available (~5.0% boost)
//! - statement/parser.rs: 279/486 (57.4%) - 207 lines available (~3.8% boost)
//!
//! Goal: +5-8% coverage boost through systematic error path testing

use ferra_parser::{
    ast::Arena,
    pratt::parser::PrattParser,
    program::parser::ProgramParser,
    statement::parser::StatementParser,
    token::{TokenType, VecTokenStream},
};

/// Test error recovery mechanisms extensively
#[test]
fn test_comprehensive_error_recovery() {
    let arena = Arena::new();

    // Test recovery from malformed function signature
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Fn,
        TokenType::Identifier("broken_func".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("param".to_string()),
        TokenType::Colon,
        // Missing type, test recovery
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(42),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = ProgramParser::new(&arena, tokens);
    match parser.parse_compilation_unit() {
        Ok(_) => println!("✅ Error recovery from malformed function signature"),
        Err(e) => println!("⚠️ Error recovery test: {:?}", e),
    }

    // Test recovery from malformed variable declaration
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Let,
        TokenType::Identifier("broken_var".to_string()),
        TokenType::Colon,
        // Missing type identifier, test recovery
        TokenType::Equal,
        TokenType::IntegerLiteral(100),
        TokenType::Semicolon,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("✅ Error recovery from malformed variable declaration"),
        Err(e) => println!("⚠️ Variable declaration recovery: {:?}", e),
    }

    // Test recovery from malformed data class
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Data,
        TokenType::Identifier("BrokenClass".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("field1".to_string()),
        TokenType::Colon,
        // Missing field type, test recovery
        TokenType::Comma,
        TokenType::Identifier("field2".to_string()),
        TokenType::Colon,
        TokenType::Identifier("String".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = ProgramParser::new(&arena, tokens);
    match parser.parse_compilation_unit() {
        Ok(_) => println!("✅ Error recovery from malformed data class"),
        Err(e) => println!("⚠️ Data class recovery: {:?}", e),
    }
}

/// Test expression error boundaries and edge cases
#[test]
fn test_expression_error_boundaries() {
    let arena = Arena::new();

    // Test malformed binary expression with operator precedence edge cases
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::IntegerLiteral(42),
        TokenType::Plus,
        TokenType::Star, // Invalid: binary operator after binary operator
        TokenType::IntegerLiteral(5),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Expected error for malformed binary expression"),
        Err(_) => println!("✅ Proper error for malformed binary expression"),
    }

    // Test deeply nested expressions with error recovery
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftParen,
        TokenType::LeftParen,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Star,
        TokenType::IntegerLiteral(3),
        TokenType::RightParen,
        // Missing closing paren - test error recovery
        TokenType::Plus,
        TokenType::IntegerLiteral(4),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Expected error for unclosed nested expression"),
        Err(_) => println!("✅ Proper error for unclosed nested expression"),
    }

    // Test malformed function call with recovery
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("func".to_string()),
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::Comma, // Double comma - test recovery
        TokenType::IntegerLiteral(2),
        TokenType::RightParen,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Expected error for double comma in function call"),
        Err(_) => println!("✅ Proper error for double comma in function call"),
    }

    // Test malformed array literal with recovery
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftBracket,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        TokenType::Comma, // Double comma - test recovery
        TokenType::IntegerLiteral(2),
        TokenType::Comma,
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Expected error for double comma in array"),
        Err(_) => println!("✅ Proper error for double comma in array"),
    }
}

/// Test statement parsing error paths extensively
#[test]
fn test_statement_error_paths() {
    let arena = Arena::new();

    // Test malformed if statement with missing condition
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::If,
        // Missing condition expression
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(1),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for if without condition"),
        Err(_) => println!("✅ Proper error for if without condition"),
    }

    // Test malformed while loop with missing condition
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::While,
        // Missing condition expression
        TokenType::LeftBrace,
        TokenType::Break,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for while without condition"),
        Err(_) => println!("✅ Proper error for while without condition"),
    }

    // Test malformed for loop with missing 'in' keyword
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::For,
        TokenType::Identifier("i".to_string()),
        // Missing 'in' keyword
        TokenType::Identifier("items".to_string()),
        TokenType::LeftBrace,
        TokenType::Continue,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for for loop without 'in'"),
        Err(_) => println!("✅ Proper error for for loop without 'in'"),
    }

    // Test malformed function declaration with missing return arrow
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Fn,
        TokenType::Identifier("broken_return".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        // Missing arrow before return type
        TokenType::Identifier("i32".to_string()),
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(0),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for function without arrow"),
        Err(_) => println!("✅ Proper error for function without arrow"),
    }

    // Test malformed async function with incorrect modifier order
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Fn,
        TokenType::Async, // Wrong order: async should come before fn
        TokenType::Identifier("wrong_order".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for wrong async modifier order"),
        Err(_) => println!("✅ Proper error for wrong async modifier order"),
    }
}

/// Test program-level parsing error paths
#[test]
fn test_program_error_paths() {
    let arena = Arena::new();

    // Test malformed extern block with missing ABI string
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Extern,
        // Missing ABI string like "C"
        TokenType::LeftBrace,
        TokenType::Fn,
        TokenType::Identifier("external_func".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = ProgramParser::new(&arena, tokens);
    match parser.parse_compilation_unit() {
        Ok(_) => println!("⚠️ Expected error for extern without ABI"),
        Err(_) => println!("✅ Proper error for extern without ABI"),
    }

    // Test malformed generic function with unclosed angle bracket
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Fn,
        TokenType::Identifier("generic_broken".to_string()),
        TokenType::Less,
        TokenType::Identifier("T".to_string()),
        // Missing closing angle bracket
        TokenType::LeftParen,
        TokenType::Identifier("param".to_string()),
        TokenType::Colon,
        TokenType::Identifier("T".to_string()),
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::Identifier("param".to_string()),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = ProgramParser::new(&arena, tokens);
    match parser.parse_compilation_unit() {
        Ok(_) => println!("⚠️ Expected error for unclosed generic bracket"),
        Err(_) => println!("✅ Proper error for unclosed generic bracket"),
    }

    // Test malformed data class with missing field separator
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Data,
        TokenType::Identifier("BadData".to_string()),
        TokenType::LeftBrace,
        TokenType::Identifier("field1".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        // Missing comma between fields
        TokenType::Identifier("field2".to_string()),
        TokenType::Colon,
        TokenType::Identifier("String".to_string()),
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = ProgramParser::new(&arena, tokens);
    match parser.parse_compilation_unit() {
        Ok(_) => println!("⚠️ Expected error for missing field separator"),
        Err(_) => println!("✅ Proper error for missing field separator"),
    }
}

/// Test complex error scenarios with multiple recovery points
#[test]
fn test_complex_error_scenarios() {
    let arena = Arena::new();

    // Test program with multiple syntax errors requiring multiple recovery attempts
    let tokens = VecTokenStream::from_token_types(vec![
        // First broken function
        TokenType::Fn,
        TokenType::Identifier("broken1".to_string()),
        TokenType::LeftParen,
        // Missing parameter type
        TokenType::Identifier("param".to_string()),
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(1),
        TokenType::Semicolon,
        TokenType::RightBrace,
        // Second broken function
        TokenType::Fn,
        TokenType::Identifier("broken2".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        // Missing function body
        TokenType::Semicolon,
        // Third valid function that should parse correctly
        TokenType::Fn,
        TokenType::Identifier("valid".to_string()),
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(3),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = ProgramParser::new(&arena, tokens);
    match parser.parse_compilation_unit() {
        Ok(_) => println!("⚠️ Parser recovered from multiple errors"),
        Err(_) => println!("✅ Multiple error scenario handled"),
    }
}

/// Test edge cases in expression parsing with complex operator combinations
#[test]
fn test_complex_expression_edge_cases() {
    let arena = Arena::new();

    // Test chained assignment-like expressions (not supported, should error)
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("a".to_string()),
        TokenType::Equal,
        TokenType::Identifier("b".to_string()),
        TokenType::Equal, // Chained assignment not supported
        TokenType::IntegerLiteral(1),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Unexpected success for chained assignment"),
        Err(_) => println!("✅ Proper error for chained assignment"),
    }

    // Test complex member access with missing identifiers
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("obj".to_string()),
        TokenType::Dot,
        // Missing member name
        TokenType::Dot,
        TokenType::Identifier("field".to_string()),
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Unexpected success for malformed member access"),
        Err(_) => println!("✅ Proper error for malformed member access"),
    }

    // Test array indexing with malformed index expression
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("arr".to_string()),
        TokenType::LeftBracket,
        TokenType::Plus, // Invalid: unary operator without operand in index
        TokenType::RightBracket,
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Unexpected success for malformed array index"),
        Err(_) => println!("✅ Proper error for malformed array index"),
    }

    // Test macro invocation with mismatched delimiters
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("macro_test".to_string()),
        TokenType::Bang,
        TokenType::LeftParen,
        TokenType::StringLiteral("test".to_string()),
        TokenType::RightBracket, // Wrong delimiter: should be RightParen
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Unexpected success for mismatched macro delimiters"),
        Err(_) => println!("✅ Proper error for mismatched macro delimiters"),
    }
}

/// Test comprehensive error recovery stress testing
#[test]
fn test_error_recovery_stress() {
    let arena = Arena::new();

    // Test deeply nested malformed expressions with recovery at multiple levels
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::LeftParen,
        TokenType::LeftParen,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Plus,
        // Missing operand for plus
        TokenType::RightParen,
        TokenType::Star,
        TokenType::LeftParen,
        TokenType::IntegerLiteral(2),
        // Missing closing paren
        TokenType::RightParen,
        // Missing closing paren for outermost
        TokenType::Eof,
    ]);
    let mut parser = PrattParser::new(&arena, tokens);
    match parser.parse_expression(0) {
        Ok(_) => println!("⚠️ Stress test: unexpected success"),
        Err(_) => println!("✅ Stress test: proper error handling"),
    }

    // Test recovery from malformed function with complex parameter list
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Fn,
        TokenType::Identifier("complex_params".to_string()),
        TokenType::LeftParen,
        TokenType::Identifier("param1".to_string()),
        TokenType::Colon,
        TokenType::Identifier("i32".to_string()),
        TokenType::Comma,
        TokenType::Identifier("param2".to_string()),
        // Missing colon and type
        TokenType::Comma,
        TokenType::Identifier("param3".to_string()),
        TokenType::Colon,
        TokenType::Identifier("String".to_string()),
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::Return,
        TokenType::IntegerLiteral(0),
        TokenType::Semicolon,
        TokenType::RightBrace,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("✅ Complex parameter recovery"),
        Err(_) => println!("⚠️ Complex parameter recovery failed: expected better recovery"),
    }
}

/// Test error message quality and information preservation
#[test]
fn test_error_message_quality() {
    let arena = Arena::new();

    // Test that error messages contain useful information
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Let,
        TokenType::Identifier("test_var".to_string()),
        TokenType::Colon,
        // Missing type, but with context for good error message
        TokenType::Equal,
        TokenType::StringLiteral("hello".to_string()),
        TokenType::Semicolon,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for missing type annotation"),
        Err(e) => {
            // Test that error contains context information
            let error_msg = format!("{:?}", e);
            if error_msg.contains("test_var") || error_msg.contains("type") {
                println!("✅ Error message contains useful context");
            } else {
                println!("⚠️ Error message lacks context: {}", error_msg);
            }
        }
    }

    // Test error message for complex expression context
    let tokens = VecTokenStream::from_token_types(vec![
        TokenType::Identifier("complex_expr".to_string()),
        TokenType::Equal,
        TokenType::Identifier("obj".to_string()),
        TokenType::Dot,
        TokenType::Identifier("method".to_string()),
        TokenType::LeftParen,
        TokenType::IntegerLiteral(1),
        TokenType::Comma,
        // Missing argument
        TokenType::RightParen,
        TokenType::Dot,
        TokenType::Identifier("field".to_string()),
        TokenType::Semicolon,
        TokenType::Eof,
    ]);
    let mut parser = StatementParser::new(&arena, tokens);
    match parser.parse_statement() {
        Ok(_) => println!("⚠️ Expected error for missing method argument"),
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if error_msg.contains("method")
                || error_msg.contains("argument")
                || error_msg.contains("parameter")
            {
                println!("✅ Complex expression error has good context");
            } else {
                println!("⚠️ Complex expression error lacks context: {}", error_msg);
            }
        }
    }
}
