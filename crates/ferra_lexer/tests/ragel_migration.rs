use ferra_lexer::{Lexer, TokenKind};

/// Test suite to ensure compatibility when migrating to Ragel DFA lexer
/// These tests verify that the current hand-written lexer produces the expected
/// token sequences that a Ragel-generated lexer should also produce.

#[test]
fn test_ragel_compatibility_basic_tokens() {
    // Test basic token recognition that Ragel should handle identically
    let src = "let x = 42 + 3.14;";
    let tokens = Lexer::new(src).lex();

    let expected_kinds = [
        TokenKind::Let,
        TokenKind::Identifier,
        TokenKind::Equal,
        TokenKind::IntegerLiteral,
        TokenKind::Plus,
        TokenKind::FloatLiteral,
        TokenKind::Semicolon,
        TokenKind::Eof,
    ];

    let actual_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(actual_kinds, expected_kinds.iter().collect::<Vec<_>>());
}

#[test]
fn test_ragel_compatibility_complex_operators() {
    // Test multi-character operators that Ragel DFA should handle efficiently
    let src = "x += y << z >> w && a || b ?? c";
    let tokens = Lexer::new(src).lex();

    let expected_kinds = [
        TokenKind::Identifier, // x
        TokenKind::PlusEqual,  // +=
        TokenKind::Identifier, // y
        TokenKind::ShiftLeft,  // <<
        TokenKind::Identifier, // z
        TokenKind::ShiftRight, // >>
        TokenKind::Identifier, // w
        TokenKind::LogicalAnd, // &&
        TokenKind::Identifier, // a
        TokenKind::LogicalOr,  // ||
        TokenKind::Identifier, // b
        TokenKind::Coalesce,   // ??
        TokenKind::Identifier, // c
        TokenKind::Eof,
    ];

    let actual_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(actual_kinds, expected_kinds.iter().collect::<Vec<_>>());
}

#[test]
fn test_ragel_compatibility_unicode_identifiers() {
    // Test Unicode identifier handling for Ragel compatibility
    let src = "let αβγ = δεζ + ηθι;";
    let tokens = Lexer::new(src).lex();

    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].lexeme, "αβγ");
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Identifier);
    assert_eq!(tokens[3].lexeme, "δεζ");
    assert_eq!(tokens[4].kind, TokenKind::Plus);
    assert_eq!(tokens[5].kind, TokenKind::Identifier);
    assert_eq!(tokens[5].lexeme, "ηθι");
    assert_eq!(tokens[6].kind, TokenKind::Semicolon);
}

#[test]
fn test_ragel_compatibility_indentation_handling() {
    // Test indentation token generation for Ragel compatibility
    let src = "a\n    b\n        c\n    d\ne";
    let tokens = Lexer::new(src).lex();

    let expected_kinds = [
        TokenKind::Identifier, // a
        TokenKind::Newline,
        TokenKind::Indent,     // for b
        TokenKind::Identifier, // b
        TokenKind::Newline,
        TokenKind::Indent,     // for c
        TokenKind::Identifier, // c
        TokenKind::Newline,
        TokenKind::Dedent,     // back to d level
        TokenKind::Identifier, // d
        TokenKind::Newline,
        TokenKind::Dedent,     // back to base level
        TokenKind::Identifier, // e
        TokenKind::Eof,
    ];

    let actual_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(actual_kinds, expected_kinds.iter().collect::<Vec<_>>());
}

#[test]
fn test_ragel_compatibility_string_literals() {
    // Test string literal parsing for Ragel compatibility
    let src = r#""hello\nworld" r"raw\string" 'c'"#;
    let tokens = Lexer::new(src).lex();

    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, r#""hello\nworld""#);

    assert_eq!(tokens[1].kind, TokenKind::RawStringLiteral);
    assert_eq!(tokens[1].lexeme, r#"r"raw\string""#);

    assert_eq!(tokens[2].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[2].lexeme, "'c'");
}

#[test]
fn test_ragel_compatibility_numeric_literals() {
    // Test numeric literal parsing for Ragel compatibility
    let src = "42 0x1A 0o77 0b1010 3.14 1e10 1_000_000";
    let tokens = Lexer::new(src).lex();

    let expected_kinds = [
        TokenKind::IntegerLiteral, // 42
        TokenKind::IntegerLiteral, // 0x1A
        TokenKind::IntegerLiteral, // 0o77
        TokenKind::IntegerLiteral, // 0b1010
        TokenKind::FloatLiteral,   // 3.14
        TokenKind::FloatLiteral,   // 1e10
        TokenKind::IntegerLiteral, // 1_000_000
        TokenKind::Eof,
    ];

    let actual_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(actual_kinds, expected_kinds.iter().collect::<Vec<_>>());
}

#[test]
fn test_ragel_compatibility_comments() {
    // Test comment handling for Ragel compatibility
    let src = "let x = 5; // line comment\nlet y = /* block */ 10;";
    let tokens = Lexer::new(src).lex();

    // Comments should be consumed, not emitted as tokens
    let expected_kinds = [
        TokenKind::Let,
        TokenKind::Identifier, // x
        TokenKind::Equal,
        TokenKind::IntegerLiteral, // 5
        TokenKind::Semicolon,
        TokenKind::Newline,
        TokenKind::Let,
        TokenKind::Identifier, // y
        TokenKind::Equal,
        TokenKind::IntegerLiteral, // 10
        TokenKind::Semicolon,
        TokenKind::Eof,
    ];

    let actual_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(actual_kinds, expected_kinds.iter().collect::<Vec<_>>());
}
