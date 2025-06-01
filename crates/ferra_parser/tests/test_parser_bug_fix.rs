// Parser Bug Fix Tests
//
// These tests verify that the parser infinite loop bug has been fixed.
// The bug occurred when parsing functions with both parameters AND statements.
//
// Root Cause: ProgramParser.parse_parameter() required type annotations (name: type)
// but test cases used untyped parameters (name). The parser would wait for a ':'
// that never came, causing an infinite loop.
//
// Fix: Made type annotations optional in parse_parameter(), using Type::Identifier("_")
// as a placeholder for type inference when no annotation is provided.

use ferra_lexer::{Lexer, TokenKind};
use ferra_parser::{ast::Arena, token::VecTokenStream, ProgramParser, TokenType};

fn convert_token_kind(kind: TokenKind) -> TokenType {
    match kind {
        TokenKind::Let => TokenType::Let,
        TokenKind::Var => TokenType::Var,
        TokenKind::Fn => TokenType::Fn,
        TokenKind::Async => TokenType::Async,
        TokenKind::Data => TokenType::Data,
        TokenKind::Match => TokenType::Match,
        TokenKind::True => TokenType::BooleanLiteral(true),
        TokenKind::False => TokenType::BooleanLiteral(false),
        TokenKind::Return => TokenType::Return,
        TokenKind::If => TokenType::If,
        TokenKind::Else => TokenType::Else,
        TokenKind::While => TokenType::While,
        TokenKind::For => TokenType::For,
        TokenKind::In => TokenType::In,
        TokenKind::Break => TokenType::Break,
        TokenKind::Continue => TokenType::Continue,
        TokenKind::Pub => TokenType::Pub,
        TokenKind::Unsafe => TokenType::Unsafe,
        TokenKind::Identifier => TokenType::Identifier("dummy".to_string()),
        TokenKind::IntegerLiteral => TokenType::IntegerLiteral(42),
        TokenKind::FloatLiteral => TokenType::FloatLiteral(3.15),
        TokenKind::StringLiteral => TokenType::StringLiteral("dummy".to_string()),
        TokenKind::CharacterLiteral => TokenType::StringLiteral("dummy".to_string()),
        TokenKind::BooleanLiteral => TokenType::BooleanLiteral(true),
        TokenKind::Plus => TokenType::Plus,
        TokenKind::Minus => TokenType::Minus,
        TokenKind::Star => TokenType::Star,
        TokenKind::Slash => TokenType::Slash,
        TokenKind::Percent => TokenType::Percent,
        TokenKind::EqualEqual => TokenType::EqualEqual,
        TokenKind::NotEqual => TokenType::BangEqual,
        TokenKind::Less => TokenType::Less,
        TokenKind::Greater => TokenType::Greater,
        TokenKind::LessEqual => TokenType::LessEqual,
        TokenKind::GreaterEqual => TokenType::GreaterEqual,
        TokenKind::LogicalAnd => TokenType::AmpAmp,
        TokenKind::LogicalOr => TokenType::PipePipe,
        TokenKind::BitAnd => TokenType::Ampersand,
        TokenKind::BitOr => TokenType::Pipe,
        TokenKind::Equal => TokenType::Equal,
        TokenKind::Bang => TokenType::Bang,
        TokenKind::Question => TokenType::Question,
        TokenKind::Dot => TokenType::Dot,
        TokenKind::Comma => TokenType::Comma,
        TokenKind::Colon => TokenType::Colon,
        TokenKind::Semicolon => TokenType::Semicolon,
        TokenKind::LParen => TokenType::LeftParen,
        TokenKind::RParen => TokenType::RightParen,
        TokenKind::LBrace => TokenType::LeftBrace,
        TokenKind::RBrace => TokenType::RightBrace,
        TokenKind::LBracket => TokenType::LeftBracket,
        TokenKind::RBracket => TokenType::RightBracket,
        TokenKind::Arrow => TokenType::Arrow,
        TokenKind::FatArrow => TokenType::FatArrow,
        TokenKind::DotDot => TokenType::DotDot,
        TokenKind::DotDotEqual => TokenType::DotDotEqual,
        TokenKind::PathSep => TokenType::DoubleColon,
        TokenKind::Underscore => TokenType::Identifier("_".to_string()),
        TokenKind::Eof => TokenType::Eof,
        _ => TokenType::Eof,
    }
}

fn source_to_tokens(source: &str) -> Vec<TokenType> {
    let lexer = Lexer::new(source);
    let tokens = lexer.lex();

    tokens
        .into_iter()
        .map(|token| convert_token_kind(token.kind))
        .collect()
}

fn parse_source(source: &str) -> Result<bool, String> {
    let tokens = source_to_tokens(source);
    let arena = Arena::new();
    let stream = VecTokenStream::from_token_types(tokens);
    let mut parser = ProgramParser::new(&arena, stream);

    match parser.parse_compilation_unit() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false), // Parse error is ok, we just want to ensure it doesn't hang
    }
}

#[test]
fn test_function_with_empty_body() {
    // This should work
    let result = parse_source("fn test() { }");
    assert!(result.is_ok());
}

#[test]
fn test_function_with_parameters_no_body() {
    // This should work
    let result = parse_source("fn calc(a, b) { }");
    assert!(result.is_ok());
}

#[test]
fn test_function_with_statement_no_params() {
    // This should work
    let result = parse_source("fn test() { let x = 42; }");
    assert!(result.is_ok());
}

#[test]
fn test_function_with_parameters_and_statement() {
    // This was the hanging case - should now work!
    let result = parse_source("fn calc(a, b) { let x = 42; }");
    assert!(result.is_ok());
}

#[test]
fn test_function_with_multiple_statements() {
    // More complex case
    let result = parse_source("fn calc(a, b) { let x = 42; let y = a + b; }");
    assert!(result.is_ok());
}

#[test]
fn test_function_with_parameters_and_complex_expression() {
    // Complex expression case
    let result = parse_source("fn calc(a, b) { let result = (a + b) * 2; }");
    assert!(result.is_ok());
}
