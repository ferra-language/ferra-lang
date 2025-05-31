use ferra_lexer::*;

// Helper function to lex the entire input string
fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_string_literal_simple() {
    let tokens = lex_all("\"hello\"");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, "\"hello\"");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("hello".to_string()))
    );
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, 7);
}

#[test]
fn test_string_literal_empty() {
    let tokens = lex_all("\"\"");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, "\"\"");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("".to_string()))
    );
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, 2);
}

#[test]
fn test_string_literal_with_escapes() {
    let input = "\"a\\nb\\tc\\\"d\\\\e\""; // raw string: "a\nb\tc\"d\\e"
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("a\nb\tc\"d\\e".to_string()))
    );
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, input.len());
}

#[test]
fn test_string_literal_unterminated_eof() {
    let input = "\"hello";
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2); // Error token + EOF
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated string literal: expected closing quote \" before end of line or file."
                .to_string()
        ))
    );
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, input.len());
}

#[test]
fn test_string_literal_unterminated_with_backslash_at_eof() {
    let input = "\"hello\\";
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated escape sequence at end of string literal: expected character after \\"
                .to_string()
        ))
    );
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, input.len());
}

#[test]
fn test_string_literal_unterminated_by_newline() {
    let input = "\"abc\ndef"; // Raw string: "abc then newline then def
    let tokens = lex_all(input);
    // Expected: Error("abc") then potentially Newline, then Identifier(def), then Eof.
    // The exact number of tokens depends on whether Newline tokens are explicitly generated
    // or if whitespace (including internal newlines) is just skipped by the main loop after an error.
    // For now, checking the first error token is key.
    assert!(
        tokens.len() >= 2,
        "Expected at least an error token and EOF"
    );

    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "\"abc");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated string literal: expected closing quote \" before end of line or file."
                .to_string()
        ))
    );
    assert_eq!(tokens[0].span.start.offset, 0);
    assert_eq!(tokens[0].span.end.offset, "\"abc".len());
}

#[test]
fn test_string_literal_invalid_escape_sequence() {
    let input = "\"invalid\\qescape\"";
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, input[..10].to_string()); // "invalid\q
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("Invalid escape sequence in string literal: \\q. Only valid escapes are \\n, \\t, \\\\, \\\" and \\u{...}.".to_string()))
    );
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[1].lexeme, "escape");
    assert_eq!(tokens[2].kind, TokenKind::Error); // stray quote
    assert_eq!(tokens[2].lexeme, "\"");
    assert_eq!(tokens[3].kind, TokenKind::Eof);
}

#[test]
fn test_string_just_escaped_quote() {
    let input = "\"\\\"\""; // raw: "\""
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("\"".to_string()))
    );
    assert_eq!(tokens[0].span.end.offset, input.len());
}

#[test]
fn test_string_multiple_escapes_and_chars() {
    let input = "\"ab\\ncd\\t ef\\\\gh\\\"ij\""; // raw: "ab\ncd\t ef\\gh\"ij"
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("ab\ncd\t ef\\gh\"ij".to_string()))
    );
    assert_eq!(tokens[0].span.end.offset, input.len());
}

#[test]
fn test_string_unicode_escapes() {
    let src = "\"\\u{41}\\u{DF}\\u{1F600}\""; // A, ß, 😀
    let tokens = lex_all(src);
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("Aß😀".to_string()))
    );

    let src_empty_braces = "\"\\u{}\"";
    let tokens_empty = lex_all(src_empty_braces);
    assert_eq!(tokens_empty[0].kind, TokenKind::Error);
    assert!(
        matches!(tokens_empty[0].literal.as_ref().unwrap(), LiteralValue::String(msg) if msg.contains("empty hex code \\u{}"))
    );

    let src_no_closing_brace = "\"\\u{41\"";
    let tokens_no_close = lex_all(src_no_closing_brace);
    assert_eq!(tokens_no_close[0].kind, TokenKind::Error);
    assert!(
        matches!(tokens_no_close[0].literal.as_ref().unwrap(), LiteralValue::String(msg) if msg.contains("unexpected character '\"' in \\u{41} sequence"))
    );

    let src_too_many_digits = "\"\\u{110000}\""; // Max is 10FFFF
    let tokens_too_many = lex_all(src_too_many_digits);
    assert_eq!(tokens_too_many[0].kind, TokenKind::Error);
    // This might be caught as invalid codepoint rather than too many digits if 110000 is parsed first
    assert!(
        matches!(tokens_too_many[0].literal.as_ref().unwrap(), LiteralValue::String(msg) if msg.contains("not a valid Unicode codepoint"))
    );

    let src_invalid_char_in_hex = "\"\\u{4G}\"";
    let tokens_invalid_hex = lex_all(src_invalid_char_in_hex);
    assert_eq!(tokens_invalid_hex[0].kind, TokenKind::Error);
    assert!(
        matches!(tokens_invalid_hex[0].literal.as_ref().unwrap(), LiteralValue::String(msg) if msg.contains("Invalid Unicode escape"))
    );

    let src_surrogate = "\"\\u{D800}\""; // Surrogate, invalid
    let tokens_surrogate = lex_all(src_surrogate);
    assert_eq!(tokens_surrogate[0].kind, TokenKind::Error);
    assert!(
        matches!(tokens_surrogate[0].literal.as_ref().unwrap(), LiteralValue::String(msg) if msg.contains("not a valid Unicode codepoint"))
    );
}

#[test]
fn test_raw_string_literal_simple() {
    let input = r#"r"hello""#;
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("hello".to_string()))
    );
}

#[test]
fn test_raw_string_literal_with_escapes_and_quotes() {
    let input = "r\"a\\n\\tb\\\\c\""; // raw string: r"a\n\tb\\c"
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("a\\n\\tb\\\\c".to_string()))
    );
}

#[test]
fn test_raw_string_literal_empty() {
    let input = r#"r"""#;
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::RawStringLiteral);
    assert_eq!(tokens[0].lexeme, input);
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String("".to_string()))
    );
}

#[test]
fn test_raw_string_literal_unterminated() {
    let input = r#"r"hello"#; // Missing closing "
    let tokens = lex_all(input);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, input);
    assert!(
        matches!(tokens[0].literal.as_ref().unwrap(), LiteralValue::String(msg) if msg.contains("Unterminated raw string literal"))
    );
}
