use ferra_lexer::*;

// Helper function to lex the entire input string
fn lex_all(input: &str) -> Vec<Token> {
    Lexer::new(input).lex()
}

#[test]
fn test_char_literal_simple() {
    let tokens = lex_all("'a'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'a'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('a')));
    assert_eq!(tokens[0].span.end.offset, 3);
}

#[test]
fn test_char_literal_escaped_newline() {
    let tokens = lex_all("'\\n'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'\\n'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('\n')));
}

#[test]
fn test_char_literal_escaped_tab() {
    let tokens = lex_all("'\\t'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'\\t'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('\t')));
}

#[test]
fn test_char_literal_escaped_carriage_return() {
    let tokens = lex_all("'\\r'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'\\r'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('\r')));
}

#[test]
fn test_char_literal_escaped_null() {
    let tokens = lex_all("'\\0'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'\\0'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('\0')));
}

#[test]
fn test_char_literal_escaped_backslash() {
    let tokens = lex_all("'\\\\'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'\\\\'");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('\\')));
}

#[test]
fn test_char_literal_escaped_single_quote() {
    let tokens = lex_all("'\\\''");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::CharacterLiteral);
    assert_eq!(tokens[0].lexeme, "'\\\''");
    assert_eq!(tokens[0].literal, Some(LiteralValue::Char('\'')));
}

#[test]
fn test_char_literal_empty() {
    let tokens = lex_all("''");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "''");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Empty character literal (in character literal)".to_string()
        ))
    );
}

#[test]
fn test_char_literal_multiple_chars() {
    let tokens = lex_all("'ab'");
    // Based on current logic: Error('a), then main loop might process 'b' and then an error for '.
    // This test focuses on the first error produced by lex_char_literal.
    assert!(
        tokens.len() >= 2,
        "Expected at least an error token and EOF"
    );
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "'a");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Multi-character literal or unterminated (in character literal)".to_string()
        ))
    );
}

#[test]
fn test_char_literal_unterminated_eof_after_opening() {
    let tokens = lex_all("'");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "'");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated character literal (EOF) (in character literal)".to_string()
        ))
    );
}

#[test]
fn test_char_literal_unterminated_eof_after_char() {
    let tokens = lex_all("'a");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "'a");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated character literal (EOF before closing quote) (in character literal)"
                .to_string()
        ))
    );
}

#[test]
fn test_char_literal_unterminated_eof_after_backslash() {
    let tokens = lex_all("'\\");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "'\\");
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Unterminated character literal after backslash (in character literal)".to_string()
        ))
    );
}

#[test]
fn test_char_literal_unterminated_by_newline() {
    let tokens = lex_all("'a\n"); // 'a then newline
                                  // Should produce an error token for unterminated char literal, then a Newline token, then EOF
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "'a"); // Consumes 'a, stops at \n
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Multi-character literal or unterminated (in character literal)".to_string()
        ))
    );
    // Optionally, check that the last token is EOF
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn test_char_literal_invalid_escape() {
    let tokens = lex_all("'\\q'");
    // Should produce an error token for the invalid escape, then a token for the stray closing quote, then EOF
    assert!(tokens.len() >= 3);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!(tokens[0].lexeme, "'\\q"); // Only up to the invalid escape
    assert_eq!(
        tokens[0].literal,
        Some(LiteralValue::String(
            "Invalid escape sequence in char literal: \\q (in character literal)".to_string()
        ))
    );
    // The next token should be the stray single quote
    assert_eq!(tokens[1].kind, TokenKind::Error); // It will be an error token for the stray '
    assert_eq!(tokens[1].lexeme, "'");
    // Optionally, check that the last token is EOF
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}
