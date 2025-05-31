use ferra_lexer::{Lexer, TokenKind};

#[test]
fn test_line_comment() {
    let src = "let x = 5; // this is a comment\nlet y = 10;";
    let tokens = Lexer::new(src).lex();
    // Comments are consumed, not emitted as tokens
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    assert_eq!(tokens[5].kind, TokenKind::Newline);
    assert_eq!(tokens[6].kind, TokenKind::Let);
}

#[test]
fn test_block_comment() {
    let src = "let x = /* comment */ 5;";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn test_unterminated_block_comment() {
    let src = "let x = /* unterminated comment";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Error);
    // Check error message
    if let Some(ferra_lexer::LiteralValue::String(msg)) = &tokens[3].literal {
        assert!(msg.contains("Unterminated block comment"));
    }
}

#[test]
fn test_unterminated_block_comment_eof() {
    let src = "/* comment without closing";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens[0].kind, TokenKind::Error);
    if let Some(ferra_lexer::LiteralValue::String(msg)) = &tokens[0].literal {
        assert!(msg.contains("Unterminated block comment"));
    }
}

#[test]
fn test_nested_block_comments() {
    let src = "let x = /* outer /* inner */ comment */ 5;";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::IntegerLiteral);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn test_unterminated_nested_block_comment() {
    let src = "/* outer /* inner comment without proper closing */";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens[0].kind, TokenKind::Error);
    if let Some(ferra_lexer::LiteralValue::String(msg)) = &tokens[0].literal {
        assert!(msg.contains("Unterminated block comment"));
    }
}

#[test]
fn test_unterminated_inner_nested_block_comment() {
    let src = "/* outer /* inner */ still in outer comment";
    let tokens = Lexer::new(src).lex();
    assert_eq!(tokens[0].kind, TokenKind::Error);
    if let Some(ferra_lexer::LiteralValue::String(msg)) = &tokens[0].literal {
        assert!(msg.contains("Unterminated block comment"));
    }
}

#[test]
fn test_multi_line_span_precision() {
    // Test multi-line unterminated block comment span precision
    let src = "let x = /*\nthis is a\nmulti-line\nunterminated comment";
    let tokens = Lexer::new(src).lex();

    // Find the error token
    let error_token = tokens.iter().find(|t| t.kind == TokenKind::Error).unwrap();

    // Verify span boundaries
    assert_eq!(error_token.span.start.line, 1); // starts on line 1
    assert_eq!(error_token.span.start.column, 9); // after "let x = "
    assert_eq!(error_token.span.start.offset, 8); // 8 characters in

    // End should be at the end of the input
    assert!(
        error_token.span.end.line > error_token.span.start.line,
        "Multi-line token should span multiple lines"
    );
    assert_eq!(error_token.span.end.line, 4); // ends on line 4
    assert_eq!(error_token.span.end.offset, src.len()); // should span to end of input

    // Verify the lexeme contains the entire unterminated comment
    assert!(error_token.lexeme.starts_with("/*"));
    assert!(error_token.lexeme.contains("multi-line"));
    assert_eq!(error_token.lexeme.len(), src.len() - 8); // everything after "let x = "
}
