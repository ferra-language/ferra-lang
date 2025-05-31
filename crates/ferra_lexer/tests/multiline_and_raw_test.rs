use ferra_lexer::*;

fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_raw_string_multiline() {
    // Test that raw strings can contain newlines (multiline)
    let input = r#"r"line 1
line 2
line 3""#;
    let tokens = lex_all(input);
    
    assert_eq!(tokens.len(), 2); // RawStringLiteral + EOF
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("line 1\nline 2\nline 3".to_string()))
    );
    
    // Verify the span covers all lines
    assert_eq!(tokens[0].span.start.line, 1);
    assert_eq!(tokens[0].span.end.line, 3);
}

#[test]
fn test_raw_string_with_backslashes() {
    // Test raw strings preserve literal backslashes (no escape processing)
    let input = r#"r"path\to\file and \n stays literal""#;
    let tokens = lex_all(input);
    
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(r"path\to\file and \n stays literal".to_string()))
    );
}

#[test]
fn test_regular_string_vs_raw_string_escapes() {
    // Compare how regular strings process escapes vs raw strings
    let regular_input = r#""hello\nworld""#;
    let raw_input = r#"r"hello\nworld""#;
    
    let regular_tokens = lex_all(regular_input);
    let raw_tokens = lex_all(raw_input);
    
    // Regular string processes escapes
    assert_eq!(regular_tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(
        regular_tokens[0].literal,
        Some(LiteralValue::String("hello\nworld".to_string()))
    );
    
    // Raw string preserves literal backslashes
    assert_eq!(raw_tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        raw_tokens[0].literal,
        Some(LiteralValue::String(r"hello\nworld".to_string()))
    );
}

#[test]
fn test_regular_string_multiline_behavior() {
    // Regular strings should NOT span multiple lines (should error at newline)
    let input = r#""hello
world""#;
    let tokens = lex_all(input);
    
    // Should produce an error token for unterminated string
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "\"hello");
    assert!(matches!(
        tokens[0].literal.as_ref().unwrap(),
        LiteralValue::String(msg) if msg.contains("Unterminated string literal")
    ));
}

#[test]
fn test_raw_string_complex_multiline() {
    // Test complex multiline raw string with various content
    let input = r#"r"function example() {
    if (x == 42) {
        return template_var;
    }
    // Comment with \backslash
}""#;
    let tokens = lex_all(input);
    
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    let expected_content = r#"function example() {
    if (x == 42) {
        return template_var;
    }
    // Comment with \backslash
}"#;
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(expected_content.to_string()))
    );
}

#[test]
fn test_raw_string_empty_lines() {
    // Test raw string with empty lines
    let input = r#"r"line 1

line 3""#;
    let tokens = lex_all(input);
    
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("line 1\n\nline 3".to_string()))
    );
}

#[test]
fn test_raw_string_precise_spans() {
    // Test that spans are precise for multiline raw strings
    let input = r#"r"a
b""#;
    let tokens = lex_all(input);
    
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, input.len());
    assert_eq!(tokens[0].span.start.line, 1);
    assert_eq!(tokens[0].span.end.line, 2);
    assert_eq!(tokens[0].span.start.column, 1);
    assert_eq!(tokens[0].span.end.column, 3); // After the closing quote
}

#[test]
fn test_mixed_string_types() {
    // Test mixing different string types in one lexing session
    let input = r#""regular" r"raw\string" 'c'"#;
    let tokens = lex_all(input);
    
    assert_eq!(tokens.len(), 4); // 3 literals + EOF
    
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].literal, Some(LiteralValue::String("regular".to_string())));
    
    assert_eq!(tokens[1].kind, TokenKind::RawStringLiteral);
    assert_eq!(tokens[1].literal, Some(LiteralValue::String(r"raw\string".to_string())));
    
    assert_eq!(tokens[2].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[2].literal, Some(LiteralValue::Char('c')));
}

#[test]
fn test_raw_string_unterminated_multiline() {
    // Test unterminated raw string across multiple lines
    let input = r#"r"this is
an unterminated
raw string"#; // Note: missing closing quote
    
    let tokens = lex_all(input);
    
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert!(matches!(
        tokens[0].literal.as_ref().unwrap(),
        LiteralValue::String(msg) if msg.contains("Unterminated raw string literal")
    ));
} 